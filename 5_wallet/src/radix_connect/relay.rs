//! Radix Connect — HTTPS relay transport.
//!
//! The wallet and a dApp exchange messages through an HTTPS relay, end-to-end encrypted with a
//! shared key derived from an X25519 key agreement (the dApp's public key is obtained during
//! pairing; the wallet uses an ephemeral key). Messages are sealed with AES-256-GCM.
//!
//! Implemented and unit-tested here: the **encryption envelope** (key agreement + AEAD
//! seal/open). The HTTP calls in [`ConnectRelay`] follow the relay's request/response shape but
//! the exact relay URL, JSON field names, KDF parameters, and CAP-21 message framing must be
//! reconciled against the official Radix Connect spec / reference relay before production use —
//! they are the spec-sensitive parts that cannot be verified without a live relay.

use deps::*;

use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
use ring::agreement::{EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519, agree_ephemeral};
use ring::digest::{SHA256, digest};
use ring::rand::{SecureRandom, SystemRandom};

use super::cap21::{WalletInteractionWire, response_to_cap21_json};
use super::{ConnectError, RelayTransport, WalletInteraction, WalletResponse};

const NONCE_LEN: usize = 12;

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn from_hex(s: &str) -> Result<Vec<u8>, ConnectError> {
    if s.len() % 2 != 0 {
        return Err(ConnectError::Transport("odd-length hex".to_string()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| ConnectError::Transport(format!("invalid hex: {e}")))
        })
        .collect()
}

/// A derived symmetric session key (AES-256) shared with one dApp connection.
pub struct SessionKey([u8; 32]);

impl SessionKey {
    /// Derives the session key from this wallet's ephemeral private key and the dApp's X25519
    /// public key (obtained during pairing). Consumes the ephemeral key (single use).
    pub fn derive(
        ephemeral_private_key: EphemeralPrivateKey,
        dapp_public_key: &[u8],
    ) -> Result<Self, ConnectError> {
        let peer = UnparsedPublicKey::new(&X25519, dapp_public_key);
        let key = agree_ephemeral(ephemeral_private_key, &peer, |shared_secret| {
            // KDF: SHA-256 of the raw shared secret. (The official protocol's exact KDF
            // — salt/info — must be confirmed against the spec.)
            let mut key = [0u8; 32];
            key.copy_from_slice(digest(&SHA256, shared_secret).as_ref());
            key
        })
        .map_err(|_| ConnectError::Transport("X25519 key agreement failed".to_string()))?;

        Ok(Self(key))
    }

    /// Encrypts `plaintext`, returning `nonce (12 bytes) || ciphertext || tag`.
    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, ConnectError> {
        let unbound = UnboundKey::new(&AES_256_GCM, &self.0)
            .map_err(|_| ConnectError::Transport("invalid AEAD key".to_string()))?;
        let key = LessSafeKey::new(unbound);

        let mut nonce_bytes = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| ConnectError::Transport("failed to generate nonce".to_string()))?;

        let mut in_out = plaintext.to_vec();
        key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::empty(),
            &mut in_out,
        )
        .map_err(|_| ConnectError::Transport("encryption failed".to_string()))?;

        let mut out = Vec::with_capacity(NONCE_LEN + in_out.len());
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&in_out);
        Ok(out)
    }

    /// Decrypts data produced by [`Self::seal`] (`nonce || ciphertext || tag`).
    pub fn open(&self, data: &[u8]) -> Result<Vec<u8>, ConnectError> {
        if data.len() < NONCE_LEN {
            return Err(ConnectError::Transport("ciphertext too short".to_string()));
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce_array: [u8; NONCE_LEN] = nonce_bytes
            .try_into()
            .map_err(|_| ConnectError::Transport("bad nonce".to_string()))?;

        let unbound = UnboundKey::new(&AES_256_GCM, &self.0)
            .map_err(|_| ConnectError::Transport("invalid AEAD key".to_string()))?;
        let key = LessSafeKey::new(unbound);

        let mut in_out = ciphertext.to_vec();
        let plaintext = key
            .open_in_place(
                Nonce::assume_unique_for_key(nonce_array),
                Aad::empty(),
                &mut in_out,
            )
            .map_err(|_| ConnectError::Transport("decryption failed".to_string()))?;

        Ok(plaintext.to_vec())
    }
}

/// Generates a fresh ephemeral X25519 key pair, returning the private key (for
/// [`SessionKey::derive`]) and the public key bytes to send to the dApp during pairing.
pub fn new_ephemeral_keypair() -> Result<(EphemeralPrivateKey, PublicKey), ConnectError> {
    let rng = SystemRandom::new();
    let private_key = EphemeralPrivateKey::generate(&X25519, &rng)
        .map_err(|_| ConnectError::Transport("failed to generate ephemeral key".to_string()))?;
    let public_key = private_key
        .compute_public_key()
        .map_err(|_| ConnectError::Transport("failed to compute public key".to_string()))?;
    Ok((private_key, public_key))
}

/// Completes the wallet side of pairing with a dApp.
///
/// Given the dApp's X25519 public key (hex, obtained from the pairing QR code / deep link) and
/// the relay coordinates, this generates the wallet's ephemeral key, derives the shared session
/// key, and returns a ready [`ConnectRelay`] plus the wallet's public key (hex) to hand back to
/// the dApp so it can derive the same session key. (The exact pairing-message envelope is the
/// spec-sensitive part; the key establishment is implemented here.)
pub fn pair(
    relay_url: String,
    session_id: String,
    dapp_public_key_hex: &str,
) -> Result<(ConnectRelay, String), ConnectError> {
    let dapp_public_key = from_hex(dapp_public_key_hex)?;
    let (ephemeral_private, ephemeral_public) = new_ephemeral_keypair()?;
    let session_key = SessionKey::derive(ephemeral_private, &dapp_public_key)?;
    let wallet_public_hex = to_hex(ephemeral_public.as_ref());
    Ok((
        ConnectRelay::new(relay_url, session_id, session_key),
        wallet_public_hex,
    ))
}

/// An HTTPS Connect Relay client.
///
/// `relay_url` is the relay endpoint, `session_id` the paired session identifier, and
/// `session_key` the derived AES key for this connection. The send/receive methods POST a JSON
/// body `{ method, sessionId, data }` — the shape used by the official
/// `@radixdlt/radix-dapp-toolkit` relay API service (`callApi` posts `{ method, sessionId, … }`).
/// Decrypted payloads are CAP-21 (see [`super::cap21`], reconciled against the official schema).
/// Remaining for live use: validating the full handshake against a running relay + real dApp.
pub struct ConnectRelay {
    pub relay_url: String,
    pub session_id: String,
    pub session_key: SessionKey,
    client: reqwest::Client,
}

impl ConnectRelay {
    pub fn new(relay_url: String, session_id: String, session_key: SessionKey) -> Self {
        Self {
            relay_url,
            session_id,
            session_key,
            client: reqwest::Client::new(),
        }
    }

    /// Encrypts and posts a wallet response payload to the relay for the dApp to pick up.
    ///
    /// `payload` is the serialized CAP-21 response (serialization TBD against spec).
    pub async fn send_encrypted(&self, payload: &[u8]) -> Result<(), ConnectError> {
        let sealed = self.session_key.seal(payload)?;
        let body = serde_json::json!({
            "method": "sendResponse",
            "sessionId": self.session_id,
            "data": to_hex(&sealed),
        });

        self.client
            .post(&self.relay_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ConnectError::Transport(e.to_string()))?;
        Ok(())
    }

    /// Polls the relay for the next encrypted request blob and decrypts it. Returns the
    /// decrypted CAP-21 request bytes (parsing into [`super::WalletInteraction`] is the
    /// spec-sensitive step left for the CAP-21 schema work).
    pub async fn receive_encrypted(&self) -> Result<Option<Vec<u8>>, ConnectError> {
        let body = serde_json::json!({
            "method": "getRequests",
            "sessionId": self.session_id,
        });

        let response = self
            .client
            .post(&self.relay_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| ConnectError::Transport(e.to_string()))?;

        let text = response
            .text()
            .await
            .map_err(|e| ConnectError::Transport(e.to_string()))?;

        if text.trim().is_empty() {
            return Ok(None);
        }

        let hex = text.trim_matches('"');
        let sealed = from_hex(hex)
            .map_err(|e| ConnectError::Transport(format!("invalid hex from relay: {e}")))?;
        let plaintext = self.session_key.open(&sealed)?;
        Ok(Some(plaintext))
    }
}

/// The live relay implements the session transport: decrypt+parse incoming CAP-21 requests, and
/// serialize+encrypt outgoing CAP-21 responses. Polls the relay until a request is available.
impl RelayTransport for ConnectRelay {
    async fn next_interaction(&mut self) -> Result<WalletInteraction, ConnectError> {
        loop {
            if let Some(plaintext) = self.receive_encrypted().await? {
                let json = String::from_utf8(plaintext)
                    .map_err(|e| ConnectError::Transport(format!("non-utf8 payload: {e}")))?;
                return WalletInteractionWire::from_json(&json)?.into_interaction();
            }
            deps::tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    async fn send_response(
        &mut self,
        interaction_id: &str,
        response: &WalletResponse,
    ) -> Result<(), ConnectError> {
        let json = response_to_cap21_json(interaction_id, response)?;
        self.send_encrypted(json.as_bytes()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_agreement_and_aead_roundtrip() {
        // Wallet and dApp each generate an ephemeral key pair.
        let (wallet_priv, wallet_pub) = new_ephemeral_keypair().unwrap();
        let (dapp_priv, dapp_pub) = new_ephemeral_keypair().unwrap();

        // Each side derives the shared session key from their private + the other's public key.
        let wallet_key = SessionKey::derive(wallet_priv, dapp_pub.as_ref()).unwrap();
        let dapp_key = SessionKey::derive(dapp_priv, wallet_pub.as_ref()).unwrap();

        // A message sealed by the wallet must open with the dApp's derived key.
        let message = b"radix connect transaction request";
        let sealed = wallet_key.seal(message).unwrap();
        assert_ne!(&sealed[NONCE_LEN..], &message[..], "must be encrypted");

        let opened = dapp_key.open(&sealed).unwrap();
        assert_eq!(opened, message);

        // Tampering with the ciphertext must fail authentication.
        let mut tampered = sealed.clone();
        *tampered.last_mut().unwrap() ^= 0xff;
        assert!(dapp_key.open(&tampered).is_err());
    }

    #[test]
    fn pairing_establishes_a_shared_session() {
        // The dApp publishes its public key (hex) in the pairing QR / deep link.
        let (dapp_priv, dapp_pub) = new_ephemeral_keypair().unwrap();
        let dapp_pub_hex = to_hex(dapp_pub.as_ref());

        // The wallet pairs with it, getting a relay client + its own public key to return.
        let (relay, wallet_pub_hex) = pair(
            "https://relay.example".to_string(),
            "session-1".to_string(),
            &dapp_pub_hex,
        )
        .unwrap();

        // The dApp derives the same session key from its private key + the wallet's public key.
        let wallet_pub = from_hex(&wallet_pub_hex).unwrap();
        let dapp_key = SessionKey::derive(dapp_priv, &wallet_pub).unwrap();

        let message = b"hello from the dapp";
        let sealed = dapp_key.seal(message).unwrap();
        let opened = relay.session_key.open(&sealed).unwrap();
        assert_eq!(opened, message);
    }
}
