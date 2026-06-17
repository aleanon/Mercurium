//! In-memory [`SecretsStore`] test double for headless verification.
//!
//! Holds the encrypted mnemonic and DB salt in memory (behind a `Mutex`) instead of the OS
//! keychain, so login/signing flows can run in CI with a known seed. Pre-seed it with
//! [`InMemorySecretsStore::seeded`] (or store into it) before booting a wallet under test.

use std::sync::Mutex;

use types::{
    AppError, Notification,
    crypto::{EncryptedMnemonic, Salt},
};

use crate::port::SecretsStore;

#[derive(Default)]
pub struct InMemorySecretsStore {
    mnemonic: Mutex<Option<EncryptedMnemonic>>,
    salt: Mutex<Option<Salt>>,
}

impl InMemorySecretsStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// A store pre-seeded with an encrypted mnemonic and DB salt (the headless-login fixture).
    pub fn seeded(mnemonic: EncryptedMnemonic, salt: Salt) -> Self {
        Self {
            mnemonic: Mutex::new(Some(mnemonic)),
            salt: Mutex::new(Some(salt)),
        }
    }
}

// Never print secret material.
impl std::fmt::Debug for InMemorySecretsStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemorySecretsStore").finish_non_exhaustive()
    }
}

fn missing(what: &str) -> AppError {
    AppError::NonFatal(Notification::Warn(format!("{what} not present in store")))
}

impl SecretsStore for InMemorySecretsStore {
    fn get_db_encryption_salt(&self) -> Result<Salt, AppError> {
        self.salt
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| missing("salt"))
    }

    fn get_encrypted_mnemonic(&self) -> Result<EncryptedMnemonic, AppError> {
        self.mnemonic
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| missing("encrypted mnemonic"))
    }

    fn store_db_encryption_salt(&self, salt: Salt) -> Result<(), AppError> {
        *self.salt.lock().unwrap() = Some(salt);
        Ok(())
    }

    fn store_encrypted_mnemonic(&self, mnemonic: &EncryptedMnemonic) -> Result<(), AppError> {
        *self.mnemonic.lock().unwrap() = Some(mnemonic.clone());
        Ok(())
    }

    fn delete_db_encryption_salt(&self) -> Result<(), AppError> {
        *self.salt.lock().unwrap() = None;
        Ok(())
    }

    fn delete_encrypted_mnemonic(&self) -> Result<(), AppError> {
        *self.mnemonic.lock().unwrap() = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salt_roundtrips_and_empty_get_errors() {
        let store = InMemorySecretsStore::new();
        assert!(store.get_db_encryption_salt().is_err(), "empty store has no salt");

        let salt = Salt::new().unwrap();
        store.store_db_encryption_salt(salt.clone()).unwrap();
        assert_eq!(
            store.get_db_encryption_salt().unwrap().to_inner(),
            salt.to_inner()
        );

        store.delete_db_encryption_salt().unwrap();
        assert!(store.get_db_encryption_salt().is_err(), "deleted salt is gone");
    }
}
