mod os_credential_store;
mod port;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

#[cfg(any(test, feature = "testing"))]
pub mod contract;

pub use os_credential_store::OsCredentialStore;
pub use port::SecretsStore;

use std::sync::Arc;
use types::{
    AppError, AppPathInner,
    crypto::{EncryptedMnemonic, Salt},
};

/// The production secrets store (OS-backed, rooted at the injected paths) as a shared trait object.
pub fn production(paths: Arc<AppPathInner>) -> Arc<dyn SecretsStore> {
    Arc::new(OsCredentialStore::new(paths))
}

/// A default-path OS store for the convenience free functions below (which have no injected paths).
fn default_store() -> Result<OsCredentialStore, AppError> {
    let paths = Arc::new(AppPathInner::new().map_err(|err| AppError::Fatal(err.to_string()))?);
    Ok(OsCredentialStore::new(paths))
}

/// Convenience free functions over the default OS-backed secrets store, for call sites that do
/// not yet inject the [`SecretsStore`] port.
pub fn get_db_encryption_salt() -> Result<Salt, AppError> {
    default_store()?.get_db_encryption_salt()
}

pub fn get_encrypted_mnemonic() -> Result<EncryptedMnemonic, AppError> {
    default_store()?.get_encrypted_mnemonic()
}

pub fn store_db_encryption_salt(salt: Salt) -> Result<(), AppError> {
    default_store()?.store_db_encryption_salt(salt)
}

pub fn store_encrypted_mnemonic(mnemonic: &EncryptedMnemonic) -> Result<(), AppError> {
    default_store()?.store_encrypted_mnemonic(mnemonic)
}

pub fn delete_salt() -> Result<(), AppError> {
    default_store()?.delete_db_encryption_salt()
}

pub fn delete_encrypted_mnemonic() -> Result<(), AppError> {
    default_store()?.delete_encrypted_mnemonic()
}
