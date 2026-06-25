//! Ledger hardware-wallet protocol layer for the Radix Babylon Ledger app.
//!
//! Signing on a Ledger never exposes the private key: the host sends an **APDU** command
//! (derivation path + payload), the device derives the key and signs *on-device*, and returns
//! the public key / signature. This module implements the host side of that protocol — the parts
//! that are pure data and therefore testable without hardware:
//!
//! - **BIP-32 path serialization** (`serialize_bip32_path`) — the Radix path is six hardened
//!   components; the device expects `count` followed by each component as big-endian `u32` with
//!   the hardened bit set.
//! - **APDU framing** (`Apdu::encode`) — `CLA | INS | P1 | P2 | Lc | data`.
//! - **Response parsing** (`parse_public_key`, `parse_signature`) — strip the trailing status
//!   word, read the fixed-size key / signature.
//!
//! The only piece that requires the physical device is the raw byte exchange, abstracted behind
//! [`LedgerTransport`]. A HID/USB implementation plugs in there; the rest is exercised by unit
//! tests against a mock transport. Instruction codes follow the Radix Babylon Ledger app
//! (`radixdlt/app-radix-babylon`); the exact opcodes and the on-device flow should be validated
//! against real hardware before release.

use deps::*;

use radix_common::crypto::{Ed25519PublicKey, Ed25519Signature};
use types::Account;

use super::FactorError;

/// Instruction-class byte for the Radix Ledger app.
const CLA: u8 = 0xAA;
/// Get the Ed25519 public key for a derivation path (P1=0: no on-device display).
const INS_GET_PUB_KEY_ED25519: u8 = 0x21;
/// Sign a transaction intent with the Ed25519 key at a derivation path.
const INS_SIGN_TX_ED25519: u8 = 0x41;
/// Sign a ROLA auth challenge with the Ed25519 key at a derivation path.
const INS_SIGN_AUTH_ED25519: u8 = 0x61;

/// The hardened-key offset (BIP-32): the high bit of the component index.
const HARDENED: u32 = 0x8000_0000;

/// Length of the trailing APDU status word (SW1 SW2).
const STATUS_LEN: usize = 2;
/// APDU success status word.
const SW_OK: u16 = 0x9000;

const PUBLIC_KEY_LEN: usize = 32;
const SIGNATURE_LEN: usize = 64;

/// Serializes a Radix derivation path (six components) for the Ledger app: a one-byte component
/// count, then each component as a big-endian `u32` with the hardened bit set.
///
/// The stored `[u32; 6]` holds the *unhardened* indices (`m/44'/1022'/net'/acct'/key'/idx'`); the
/// device expects them hardened, so we OR in [`HARDENED`].
pub fn serialize_bip32_path(path: &[u32; 6]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + path.len() * 4);
    out.push(path.len() as u8);
    for component in path {
        out.extend_from_slice(&(component | HARDENED).to_be_bytes());
    }
    out
}

/// An APDU command for the Radix Ledger app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Apdu {
    pub ins: u8,
    pub p1: u8,
    pub p2: u8,
    pub data: Vec<u8>,
}

impl Apdu {
    /// `GET_PUB_KEY_ED25519` for a derivation path.
    pub fn get_public_key(path: &[u32; 6]) -> Self {
        Self { ins: INS_GET_PUB_KEY_ED25519, p1: 0, p2: 0, data: serialize_bip32_path(path) }
    }

    /// `SIGN_TX_ED25519`: the path followed by the transaction-intent bytes to sign.
    pub fn sign_transaction(path: &[u32; 6], intent: &[u8]) -> Self {
        let mut data = serialize_bip32_path(path);
        data.extend_from_slice(intent);
        Self { ins: INS_SIGN_TX_ED25519, p1: 0, p2: 0, data }
    }

    /// `SIGN_AUTH_ED25519`: the path followed by the ROLA challenge payload to sign.
    pub fn sign_auth(path: &[u32; 6], payload: &[u8]) -> Self {
        let mut data = serialize_bip32_path(path);
        data.extend_from_slice(payload);
        Self { ins: INS_SIGN_AUTH_ED25519, p1: 0, p2: 0, data }
    }

    /// Encodes the command as raw APDU bytes: `CLA | INS | P1 | P2 | Lc | data`.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(5 + self.data.len());
        out.push(CLA);
        out.push(self.ins);
        out.push(self.p1);
        out.push(self.p2);
        out.push(self.data.len() as u8);
        out.extend_from_slice(&self.data);
        out
    }
}

/// Splits a device response into its payload and status word, returning the payload only on a
/// `0x9000` success status.
fn check_status(response: &[u8]) -> Result<&[u8], FactorError> {
    if response.len() < STATUS_LEN {
        return Err(FactorError::Unavailable("Ledger response too short".to_string()));
    }
    let (payload, status) = response.split_at(response.len() - STATUS_LEN);
    let sw = u16::from_be_bytes([status[0], status[1]]);
    if sw != SW_OK {
        return Err(FactorError::Unavailable(format!("Ledger returned status {sw:#06x}")));
    }
    Ok(payload)
}

/// Parses a `GET_PUB_KEY` response: a 32-byte Ed25519 public key (after the status check).
pub fn parse_public_key(response: &[u8]) -> Result<Ed25519PublicKey, FactorError> {
    let payload = check_status(response)?;
    let key = payload
        .get(..PUBLIC_KEY_LEN)
        .ok_or_else(|| FactorError::Unavailable("public key truncated".to_string()))?;
    let bytes: [u8; PUBLIC_KEY_LEN] =
        key.try_into().map_err(|_| FactorError::Unavailable("public key length".to_string()))?;
    Ok(Ed25519PublicKey(bytes))
}

/// Parses a `SIGN` response: a 64-byte Ed25519 signature (after the status check).
pub fn parse_signature(response: &[u8]) -> Result<Ed25519Signature, FactorError> {
    let payload = check_status(response)?;
    let sig = payload
        .get(..SIGNATURE_LEN)
        .ok_or_else(|| FactorError::Unavailable("signature truncated".to_string()))?;
    let bytes: [u8; SIGNATURE_LEN] =
        sig.try_into().map_err(|_| FactorError::Unavailable("signature length".to_string()))?;
    Ok(Ed25519Signature(bytes))
}

/// The physical link to a Ledger device. The HID/USB implementation lives outside this crate (it
/// needs system USB libraries); this trait keeps the protocol logic above pure and testable.
pub trait LedgerTransport: Send + Sync {
    /// Sends an encoded APDU and returns the raw response (payload + status word).
    fn exchange(&self, apdu: &[u8]) -> Result<Vec<u8>, FactorError>;
}

/// Derives the Radix Ledger derivation path for an account as the six hardened components.
fn account_path(account: &Account) -> [u32; 6] {
    account.derivation_path()
}

/// Requests the account's public key from a connected Ledger.
pub fn ledger_account_public_key(
    transport: &dyn LedgerTransport,
    account: &Account,
) -> Result<Ed25519PublicKey, FactorError> {
    let apdu = Apdu::get_public_key(&account_path(account)).encode();
    let response = transport.exchange(&apdu)?;
    parse_public_key(&response)
}

/// Asks a connected Ledger to sign a transaction intent for the account.
pub fn ledger_sign_transaction(
    transport: &dyn LedgerTransport,
    account: &Account,
    intent: &[u8],
) -> Result<Ed25519Signature, FactorError> {
    let apdu = Apdu::sign_transaction(&account_path(account), intent).encode();
    let response = transport.exchange(&apdu)?;
    parse_signature(&response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_hardened_path_with_count_prefix() {
        let path = [44, 1022, 1, 0, 1460, 0];
        let bytes = serialize_bip32_path(&path);
        assert_eq!(bytes[0], 6, "count prefix");
        assert_eq!(bytes.len(), 1 + 6 * 4);
        // First component: 44 hardened = 0x8000002C, big-endian.
        assert_eq!(&bytes[1..5], &[0x80, 0x00, 0x00, 0x2C]);
        // Second component: 1022 hardened = 0x800003FE.
        assert_eq!(&bytes[5..9], &[0x80, 0x00, 0x03, 0xFE]);
    }

    #[test]
    fn apdu_framing_has_cla_ins_and_length() {
        let apdu = Apdu::get_public_key(&[44, 1022, 1, 0, 1460, 0]);
        let raw = apdu.encode();
        assert_eq!(raw[0], CLA);
        assert_eq!(raw[1], INS_GET_PUB_KEY_ED25519);
        assert_eq!(raw[2], 0); // P1
        assert_eq!(raw[3], 0); // P2
        assert_eq!(raw[4] as usize, raw.len() - 5); // Lc == data length
    }

    #[test]
    fn sign_apdu_appends_payload_after_path() {
        let path = [44, 1022, 1, 0, 1460, 0];
        let payload = [0xDE, 0xAD, 0xBE, 0xEF];
        let apdu = Apdu::sign_auth(&path, &payload);
        assert_eq!(apdu.ins, INS_SIGN_AUTH_ED25519);
        // data = serialized path (25 bytes) followed by the payload.
        assert_eq!(apdu.data.len(), 25 + payload.len());
        assert_eq!(&apdu.data[25..], &payload);
    }

    #[test]
    fn parse_rejects_non_ok_status() {
        // 32 zero bytes + a 0x6985 (denied) status word.
        let mut response = vec![0u8; PUBLIC_KEY_LEN];
        response.extend_from_slice(&[0x69, 0x85]);
        assert!(parse_public_key(&response).is_err());
    }

    #[test]
    fn parse_reads_public_key_on_ok_status() {
        let mut response = vec![7u8; PUBLIC_KEY_LEN];
        response.extend_from_slice(&SW_OK.to_be_bytes());
        let key = parse_public_key(&response).unwrap();
        assert_eq!(key.0, [7u8; PUBLIC_KEY_LEN]);
    }

    #[test]
    fn parse_signature_reads_64_bytes_on_ok() {
        let mut response = vec![3u8; SIGNATURE_LEN];
        response.extend_from_slice(&SW_OK.to_be_bytes());
        let sig = parse_signature(&response).unwrap();
        assert_eq!(sig.0, [3u8; SIGNATURE_LEN]);
    }

    /// A mock transport that echoes a canned response, proving the host-side flow end to end
    /// without hardware.
    struct MockTransport(Vec<u8>);
    impl LedgerTransport for MockTransport {
        fn exchange(&self, apdu: &[u8]) -> Result<Vec<u8>, FactorError> {
            assert_eq!(apdu[0], CLA, "transport receives a framed APDU");
            Ok(self.0.clone())
        }
    }

    #[test]
    fn end_to_end_public_key_via_mock_transport() {
        use bip39::{Language, Mnemonic};
        use types::Network;
        let mnemonic = Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap();
        let account = crate::wallet::create_account_from_mnemonic(
            &mnemonic,
            None,
            0,
            0,
            "Acct".to_string(),
            Network::Stokenet,
        );
        let mut canned = vec![9u8; PUBLIC_KEY_LEN];
        canned.extend_from_slice(&SW_OK.to_be_bytes());
        let transport = MockTransport(canned);
        let key = ledger_account_public_key(&transport, &account).unwrap();
        assert_eq!(key.0, [9u8; PUBLIC_KEY_LEN]);
    }
}
