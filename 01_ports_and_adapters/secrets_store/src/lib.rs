mod os_credential_store;
mod port;

pub use os_credential_store::OsCredentialStore;
pub use port::SecretsStore;

use types::{
    AppError,
    crypto::{EncryptedMnemonic, Salt},
};

/// Convenience free functions over the default OS-backed secrets store, for call sites that do
/// not yet inject the [`SecretsStore`] port.
pub fn get_db_encryption_salt() -> Result<Salt, AppError> {
    OsCredentialStore.get_db_encryption_salt()
}

pub fn get_encrypted_mnemonic() -> Result<EncryptedMnemonic, AppError> {
    OsCredentialStore.get_encrypted_mnemonic()
}

pub fn store_db_encryption_salt(salt: Salt) -> Result<(), AppError> {
    OsCredentialStore.store_db_encryption_salt(salt)
}

pub fn store_encrypted_mnemonic(mnemonic: &EncryptedMnemonic) -> Result<(), AppError> {
    OsCredentialStore.store_encrypted_mnemonic(mnemonic)
}

pub fn delete_salt() -> Result<(), AppError> {
    OsCredentialStore.delete_db_encryption_salt()
}

pub fn delete_encrypted_mnemonic() -> Result<(), AppError> {
    OsCredentialStore.delete_encrypted_mnemonic()
}
