//! Factor sources — the abstraction over *how* an account is signed.
//!
//! Today the wallet has a single on-device seed factor; the official wallet supports multiple
//! (Ledger, off-device mnemonic, Arculus, password) composed into Security Shields. The
//! [`SigningFactor`] trait is the seam: call sites sign through it, so adding a Ledger or another
//! factor doesn't change the transaction/ROLA code. [`FactorSourceKind`] (in `types`) is the
//! catalogue of factor kinds.

pub mod access_controller;
pub mod ledger;
pub mod multi_factor;

use deps::*;

use bip39::Mnemonic;
use radix_common::crypto::{Ed25519PublicKey, Ed25519Signature, Hash};
use types::{Account, FactorSourceKind, crypto::Ed25519KeyPair};

use crate::wallet::signing_keypair_for_account;

#[derive(Debug, thiserror::Error)]
pub enum FactorError {
    #[error("Factor source unavailable: {0}")]
    Unavailable(String),
}

/// A factor source's signing capability for an account.
pub trait SigningFactor {
    fn kind(&self) -> FactorSourceKind;

    /// The public key for the account's derivation path (used to identify the signer to dApps).
    fn account_public_key(&self, account: &Account) -> Result<Ed25519PublicKey, FactorError>;

    /// Sign a pre-computed hash (transaction-intent or ROLA challenge) with the account's key.
    fn sign_account(
        &self,
        account: &Account,
        hash: &Hash,
    ) -> Result<Ed25519Signature, FactorError>;
}

/// The on-device seed factor: derives keys from the wallet mnemonic held in the OS secrets store.
pub struct DeviceFactor {
    mnemonic: Mnemonic,
    seed_password: Option<String>,
}

impl DeviceFactor {
    pub fn new(mnemonic: Mnemonic, seed_password: Option<String>) -> Self {
        Self { mnemonic, seed_password }
    }

    fn keypair(&self, account: &Account) -> Ed25519KeyPair {
        signing_keypair_for_account(&self.mnemonic, self.seed_password.as_deref(), account)
    }
}

impl SigningFactor for DeviceFactor {
    fn kind(&self) -> FactorSourceKind {
        FactorSourceKind::Device
    }

    fn account_public_key(&self, account: &Account) -> Result<Ed25519PublicKey, FactorError> {
        Ok(self.keypair(account).radixdlt_public_key())
    }

    fn sign_account(
        &self,
        account: &Account,
        hash: &Hash,
    ) -> Result<Ed25519Signature, FactorError> {
        Ok(self.keypair(account).sign_hash(hash))
    }
}

/// A Ledger hardware factor source.
///
/// The protocol (APDU framing, BIP-32 path serialization, response parsing) is implemented in
/// [`ledger`] and exercised by tests against a mock transport. The private key never leaves the
/// device: this factor builds the APDU, hands it to a [`LedgerTransport`], and parses the reply.
/// The transport is the only hardware-dependent piece (raw USB/HID byte exchange); with no
/// transport connected the factor reports [`FactorError::Unavailable`].
///
/// Note: the Radix Ledger app signs a *transaction intent*, not a pre-hashed digest, so
/// [`SigningFactor::sign_account`] (which receives a [`Hash`]) is not the device's signing entry
/// point — use [`ledger::ledger_sign_transaction`] with the intent bytes for on-device signing.
pub struct LedgerFactor {
    pub device_label: String,
    pub transport: Option<Box<dyn ledger::LedgerTransport>>,
}

impl LedgerFactor {
    /// A Ledger factor with no transport connected (the headless default).
    pub fn disconnected(device_label: impl Into<String>) -> Self {
        Self { device_label: device_label.into(), transport: None }
    }

    /// A Ledger factor backed by a connected transport (e.g. a HID/USB link).
    pub fn connected(
        device_label: impl Into<String>,
        transport: Box<dyn ledger::LedgerTransport>,
    ) -> Self {
        Self { device_label: device_label.into(), transport: Some(transport) }
    }

    fn transport(&self) -> Result<&dyn ledger::LedgerTransport, FactorError> {
        self.transport
            .as_deref()
            .ok_or_else(|| FactorError::Unavailable("no Ledger transport connected".to_string()))
    }

    /// Signs a transaction intent on the device (the Radix Ledger app's native signing flow).
    pub fn sign_transaction(
        &self,
        account: &Account,
        intent: &[u8],
    ) -> Result<Ed25519Signature, FactorError> {
        ledger::ledger_sign_transaction(self.transport()?, account, intent)
    }
}

impl SigningFactor for LedgerFactor {
    fn kind(&self) -> FactorSourceKind {
        FactorSourceKind::LedgerHardware
    }

    fn account_public_key(&self, account: &Account) -> Result<Ed25519PublicKey, FactorError> {
        ledger::ledger_account_public_key(self.transport()?, account)
    }

    fn sign_account(
        &self,
        _account: &Account,
        _hash: &Hash,
    ) -> Result<Ed25519Signature, FactorError> {
        // The Radix Ledger app signs intents, not bare hashes; route on-device signing through
        // `sign_transaction` with the intent bytes instead.
        Err(FactorError::Unavailable(
            "Ledger signs transaction intents, not pre-hashed digests; use sign_transaction"
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_common::crypto::{hash, verify_ed25519};
    use types::Network;

    fn mnemonic() -> Mnemonic {
        use bip39::Language;
        Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap()
    }

    fn account() -> Account {
        crate::wallet::create_account_from_mnemonic(&mnemonic(), None, 0, 0, "Acct".to_string(), Network::Stokenet)
    }

    #[test]
    fn device_factor_signs_and_public_key_verifies() {
        let factor = DeviceFactor::new(mnemonic(), None);
        let account = account();

        let public_key = factor.account_public_key(&account).unwrap();
        assert_eq!(public_key, account.public_key);
        assert_eq!(factor.kind(), FactorSourceKind::Device);

        let message_hash = hash(b"intent");
        let signature = factor.sign_account(&account, &message_hash).unwrap();
        assert!(verify_ed25519(message_hash, &public_key, &signature));
    }

    #[test]
    fn ledger_factor_is_unavailable_without_transport() {
        let ledger = LedgerFactor::disconnected("Nano");
        assert_eq!(ledger.kind(), FactorSourceKind::LedgerHardware);
        assert!(ledger.account_public_key(&account()).is_err());
        assert!(ledger.sign_account(&account(), &radix_common::crypto::hash(b"x")).is_err());
        assert!(ledger.sign_transaction(&account(), b"intent").is_err());
    }
}
