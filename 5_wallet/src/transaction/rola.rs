//! ROLA — Radix Off-Ledger Authentication.
//!
//! ROLA lets a dApp verify that the wallet controls a given account or persona (identity)
//! without an on-ledger transaction. The wallet signs a challenge issued by the dApp; the
//! dApp's backend verifies the signature and that the public key maps to the claimed address,
//! cross-checking the dApp definition via its `.well-known/radix.json`.
//!
//! The signed message is the Blake2b-256 hash (Radix's `hash`) of:
//! ```text
//! 0x52 ("R") || challenge (32 bytes) || len(dApp_def_address) (1 byte)
//!            || dApp_def_address (bech32m string bytes) || origin (string bytes)
//! ```
//!
//! NOTE: the byte layout below follows the published ROLA construction. Before relying on it
//! against production dApps, cross-check against the official Radix reference test vectors
//! (radixdlt-scrypto / sargon), as ROLA verification is byte-exact.

use deps::*;

use radix_common::crypto::{Ed25519PublicKey, Ed25519Signature, hash};

use types::crypto::Ed25519KeyPair;

/// Single-byte domain-separation prefix for ROLA payloads (ASCII 'R').
pub const ROLA_PAYLOAD_PREFIX: u8 = 0x52;

/// A 32-byte challenge issued by a dApp.
pub type RolaChallenge = [u8; 32];

/// A signed ROLA challenge, in the shape a dApp expects (public key + signature + curve).
#[derive(Debug, Clone)]
pub struct SignedRolaChallenge {
    pub public_key: Ed25519PublicKey,
    pub signature: Ed25519Signature,
}

/// Builds the exact byte payload that gets hashed and signed for ROLA.
pub fn rola_payload(
    challenge: &RolaChallenge,
    dapp_definition_address: &str,
    origin: &str,
) -> Vec<u8> {
    let mut payload =
        Vec::with_capacity(1 + 32 + 1 + dapp_definition_address.len() + origin.len());
    payload.push(ROLA_PAYLOAD_PREFIX);
    payload.extend_from_slice(challenge);
    // dApp definition address length fits in one byte (radix addresses are well under 255 chars).
    payload.push(dapp_definition_address.len() as u8);
    payload.extend_from_slice(dapp_definition_address.as_bytes());
    payload.extend_from_slice(origin.as_bytes());
    payload
}

/// Signs a ROLA challenge with the given key pair (account or identity/persona key).
pub fn sign_rola_challenge(
    keypair: &Ed25519KeyPair,
    challenge: &RolaChallenge,
    dapp_definition_address: &str,
    origin: &str,
) -> SignedRolaChallenge {
    let payload = rola_payload(challenge, dapp_definition_address, origin);
    let message_hash = hash(payload);

    SignedRolaChallenge {
        public_key: keypair.radix_private_key().public_key(),
        signature: keypair.sign_hash(&message_hash),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_common::crypto::verify_ed25519;
    use types::{
        Network,
        crypto::{Bip32Entity, Bip32KeyKind},
    };

    fn identity_keypair() -> Ed25519KeyPair {
        use deps::bip39::{Language, Mnemonic};
        let mnemonic = Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap();
        Ed25519KeyPair::new(
            &mnemonic,
            None,
            0,
            Network::Mainnet,
            Bip32Entity::Identity,
            Bip32KeyKind::TransactionSigning,
        )
        .0
    }

    #[test]
    fn payload_has_expected_layout() {
        let challenge: RolaChallenge = [7u8; 32];
        let dapp = "account_rdx12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66";
        let origin = "https://dashboard.radixdlt.com";

        let payload = rola_payload(&challenge, dapp, origin);

        assert_eq!(payload[0], ROLA_PAYLOAD_PREFIX);
        assert_eq!(&payload[1..33], &challenge[..]);
        assert_eq!(payload[33] as usize, dapp.len());
        assert_eq!(&payload[34..34 + dapp.len()], dapp.as_bytes());
        assert_eq!(&payload[34 + dapp.len()..], origin.as_bytes());
    }

    #[test]
    fn signs_and_verifies_challenge() {
        let keypair = identity_keypair();
        let challenge: RolaChallenge = [42u8; 32];
        let dapp = "account_rdx12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66";
        let origin = "https://example.com";

        let signed = sign_rola_challenge(&keypair, &challenge, dapp, origin);

        // The dApp would recompute the same payload hash and verify against the public key.
        let payload = rola_payload(&challenge, dapp, origin);
        let message_hash = hash(payload);
        assert!(verify_ed25519(
            message_hash,
            &signed.public_key,
            &signed.signature
        ));

        // A tampered origin must not verify.
        let tampered = rola_payload(&challenge, dapp, "https://evil.com");
        let tampered_hash = hash(tampered);
        assert!(!verify_ed25519(
            tampered_hash,
            &signed.public_key,
            &signed.signature
        ));
    }
}
