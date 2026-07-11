//! OS-backed [`SecretsStore`] adapter.
//!
//! On Windows the secrets live in the Windows Credential Manager; on Unix they live as files in
//! the app config directory. Moved here from the former `handles::credentials` module.

use deps::*;

use std::sync::Arc;

use types::{
    AppError, AppPathInner,
    crypto::{EncryptedMnemonic, Salt},
};
use zeroize::Zeroize;

use crate::port::SecretsStore;

#[cfg(windows)]
const SALT_TARGET_NAME: &'static str = "l4h4c5aPo1ULu3dLQjCYrq2TJNY3wZiYwGL4jTOZ1Lk=";
#[cfg(windows)]
const ENCRYPTED_MNEMONIC_TARGET_NAME: &'static str = "Bk3oMH8tphurhYE3b/U/a4k03oefVrATNCFvWKz6FxA=";

#[cfg(unix)]
const SALT_TARGET_NAME: &str = "db_salt.json";
#[cfg(unix)]
const ENCRYPTED_MNEMONIC_TARGET_NAME: &str = "mnemonic.json";

/// The default OS-backed secrets store. Holds the injected [`AppPathInner`] (used on Unix to
/// locate the config directory; ignored on Windows, which uses the Credential Manager).
#[derive(Debug, Clone)]
pub struct OsCredentialStore {
    paths: Arc<AppPathInner>,
}

impl OsCredentialStore {
    pub fn new(paths: Arc<AppPathInner>) -> Self {
        Self { paths }
    }
}

impl SecretsStore for OsCredentialStore {
    fn get_db_encryption_salt(&self) -> Result<Salt, AppError> {
        let blob = platform::get_blob(&self.paths, SALT_TARGET_NAME)?;
        blob.try_into()
            .map_err(|err| AppError::Fatal(format!("Failed to get Salt: {err}")))
    }

    fn get_encrypted_mnemonic(&self) -> Result<EncryptedMnemonic, AppError> {
        let blob = platform::get_blob(&self.paths, ENCRYPTED_MNEMONIC_TARGET_NAME)?;
        serde_json::from_slice(&blob).map_err(|err| {
            AppError::Fatal(format!("Failed to parse blob to Encrypted Mnemonic: {err}"))
        })
    }

    fn store_db_encryption_salt(&self, salt: Salt) -> Result<(), AppError> {
        let mut salt = salt.to_inner();
        let result = platform::store_blob(&self.paths, &salt, SALT_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to store salt, {err}")));
        salt.zeroize();
        result
    }

    fn store_encrypted_mnemonic(&self, mnemonic: &EncryptedMnemonic) -> Result<(), AppError> {
        let mut blob = serde_json::to_vec(mnemonic)
            .map_err(|err| AppError::Fatal(format!("Failed to parse Encrypted Mnemonic, {err}")))?;
        let result = platform::store_blob(&self.paths, &blob, ENCRYPTED_MNEMONIC_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to save EncryptedMnemonic, {err}")));
        blob.zeroize();
        result
    }

    fn delete_db_encryption_salt(&self) -> Result<(), AppError> {
        platform::delete_blob(&self.paths, SALT_TARGET_NAME)
    }

    fn delete_encrypted_mnemonic(&self) -> Result<(), AppError> {
        platform::delete_blob(&self.paths, ENCRYPTED_MNEMONIC_TARGET_NAME)
    }
}

#[cfg(unix)]
mod platform {
    use super::*;
    use types::Notification;

    pub fn get_blob(paths: &AppPathInner, target_name: &str) -> Result<Vec<u8>, AppError> {
        let mut config_file = paths.config_directory();
        config_file.push(target_name);
        std::fs::read(config_file)
            .map_err(|err| AppError::NonFatal(Notification::Warn(err.to_string())))
    }

    pub fn store_blob(
        paths: &AppPathInner,
        blob: &[u8],
        target_name: &str,
    ) -> Result<(), AppError> {
        let mut config_file = paths.config_directory();
        config_file.push(target_name);
        std::fs::write(config_file, blob)
            .map_err(|err| AppError::NonFatal(Notification::Warn(err.to_string())))
    }

    pub fn delete_blob(paths: &AppPathInner, target_name: &str) -> Result<(), AppError> {
        let mut config_file = paths.config_directory();
        config_file.push(target_name);
        std::fs::remove_file(config_file)
            .map_err(|err| AppError::NonFatal(Notification::Warn(err.to_string())))
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use windows::{
        Win32::{
            Foundation::E_POINTER,
            Security::Credentials::{
                CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree,
                CredReadW, CredWriteW,
            },
        },
        core::{PCWSTR, PWSTR},
    };

    pub fn get_blob(
        _paths: &types::AppPathInner,
        target_name: &str,
    ) -> Result<Vec<u8>, AppError> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();
        target_name.push(0);

        let mut cred_ptr: *mut CREDENTIALW = std::ptr::null_mut();

        let result = unsafe {
            CredReadW(
                PCWSTR(target_name.as_ptr()),
                CRED_TYPE_GENERIC,
                None,
                &mut cred_ptr,
            )
            .and_then(|_| {
                if !cred_ptr.is_null() {
                    let cred = &*cred_ptr;
                    let slice = std::slice::from_raw_parts(
                        cred.CredentialBlob,
                        cred.CredentialBlobSize as usize / 2,
                    );
                    let bytes = slice.to_vec();
                    CredFree(cred_ptr as *mut _);
                    Ok(bytes)
                } else {
                    Err(windows::core::Error::new(
                        E_POINTER,
                        "Null pointer received for credentials",
                    ))
                }
            })
        };

        result.map_err(|err| AppError::Fatal(format!("Failed to get credentials blob: {err}")))
    }

    pub fn store_blob(
        _paths: &types::AppPathInner,
        blob: &[u8],
        target_name: &str,
    ) -> Result<(), AppError> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();
        target_name.push(0);
        let mut blob = blob.to_vec();

        let result = unsafe {
            let credentials = CREDENTIALW {
                Type: CRED_TYPE_GENERIC,
                TargetName: PWSTR(target_name.as_mut_ptr()),
                CredentialBlob: blob.as_mut_ptr(),
                CredentialBlobSize: (blob.len() * 2) as u32,
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                ..Default::default()
            };
            CredWriteW(&credentials, 0)
        };

        result.map_err(|err| AppError::Fatal(err.to_string()))
    }

    pub fn delete_blob(_paths: &types::AppPathInner, target_name: &str) -> Result<(), AppError> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();
        target_name.push(0);

        unsafe { CredDeleteW(PCWSTR(target_name.as_mut_ptr()), CRED_TYPE_GENERIC, None) }
            .map_err(|err| AppError::Fatal(err.to_string()))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn salt_store_get_delete_roundtrip() {
        let paths = Arc::new(AppPathInner::new().unwrap());
        paths.create_directories_if_not_exists().ok();

        let store = OsCredentialStore::new(paths);
        let salt = Salt::new().unwrap();

        store.store_db_encryption_salt(salt.clone()).unwrap();
        let loaded = store.get_db_encryption_salt().unwrap();
        assert_eq!(salt.to_inner(), loaded.to_inner());

        store.delete_db_encryption_salt().unwrap();
        assert!(store.get_db_encryption_salt().is_err());
    }
}
