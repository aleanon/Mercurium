//! Shared [`SecretsStore`] contract suite.
//!
//! One set of behavioural assertions that every `SecretsStore` implementation
//! must satisfy, run against both the in-memory fake and the real OS-backed
//! adapter. This binds the fake to reality: if the fake accepts a sequence the
//! real store would reject (or vice versa), the shared suite catches it, so the
//! headless tests that rely on the fake stay trustworthy.

use types::crypto::{EncryptedMnemonic, Password, Salt, bip39};

use crate::SecretsStore;

fn sample_mnemonic() -> (EncryptedMnemonic, Password) {
    let mnemonic = bip39::Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
    let password = Password::from("contract-suite-password");
    let encrypted = EncryptedMnemonic::new(&mnemonic, "", &password).unwrap();
    (encrypted, password)
}

/// Exercises the full port contract against `store`. `store` must start empty
/// and is left empty on success. Panics with a descriptive message on any
/// contract violation.
pub fn assert_secrets_store_contract<S: SecretsStore>(store: &S) {
    // Empty store: reads fail rather than returning stale/blank data.
    assert!(
        store.get_db_encryption_salt().is_err(),
        "empty store must not return a salt"
    );
    assert!(
        store.get_encrypted_mnemonic().is_err(),
        "empty store must not return a mnemonic"
    );

    // Salt: store -> get (round-trips) -> delete -> get (gone).
    let salt = Salt::new().unwrap();
    store.store_db_encryption_salt(salt.clone()).unwrap();
    assert_eq!(
        store.get_db_encryption_salt().unwrap().to_inner(),
        salt.to_inner(),
        "salt must round-trip byte-for-byte"
    );
    store.delete_db_encryption_salt().unwrap();
    assert!(
        store.get_db_encryption_salt().is_err(),
        "deleted salt must be gone"
    );

    // Mnemonic: store -> get -> decrypt both -> same phrase; then delete -> gone.
    let (encrypted, password) = sample_mnemonic();
    let expected_phrase = encrypted.decrypt_mnemonic(&password).unwrap().0.phrase().to_string();

    store.store_encrypted_mnemonic(&encrypted).unwrap();
    let loaded = store.get_encrypted_mnemonic().unwrap();
    let loaded_phrase = loaded.decrypt_mnemonic(&password).unwrap().0.phrase().to_string();
    assert_eq!(
        loaded_phrase, expected_phrase,
        "stored mnemonic must decrypt to the same phrase after a round-trip"
    );

    store.delete_encrypted_mnemonic().unwrap();
    assert!(
        store.get_encrypted_mnemonic().is_err(),
        "deleted mnemonic must be gone"
    );
}
