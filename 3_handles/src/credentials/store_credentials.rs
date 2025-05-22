use deps::*;

use super::{ENCRYPTED_MNEMONIC_TARGET_NAME, SALT_TARGET_NAME};
use types::{AppError, crypto::Salt};

#[cfg(windows)]
pub use mswindows::*;

#[cfg(windows)]
pub mod mswindows {
    use super::*;
    use types::crypto::EncryptedMnemonic;
    use windows::{
        Win32::Security::Credentials::{
            CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredWriteW,
        },
        core::PWSTR,
    };
    use zeroize::Zeroize;

    pub fn store_db_encryption_salt(salt: Salt) -> Result<(), AppError> {
        let mut salt = salt.to_inner();

        store_blob(salt.as_mut_ptr(), salt.len(), SALT_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to store salt, {}", err)))
    }

    pub fn store_encrypted_mnemonic(
        encrypted_mnemonic: &EncryptedMnemonic,
    ) -> Result<(), AppError> {
        let mut blob = serde_json::to_vec(encrypted_mnemonic).map_err(|err| {
            AppError::Fatal(format!("Failed to parse Encrypted Mnemonic, {}", err))
        })?;

        store_blob(
            blob.as_mut_ptr(),
            blob.len(),
            ENCRYPTED_MNEMONIC_TARGET_NAME,
        )
        .map_err(|err| AppError::Fatal(format!("Failed to save EnctyptedMnemonic, {}", err)))?;

        blob.zeroize();

        Ok(())
    }

    fn store_blob(
        blob: *mut u8,
        blob_length: usize,
        target_name: &str,
    ) -> windows::core::Result<()> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();
        // Pushes 0 to the vector to make it null terminated C compatible
        target_name.push(0);

        unsafe {
            let credentials = CREDENTIALW {
                Type: CRED_TYPE_GENERIC,
                TargetName: PWSTR(target_name.as_mut_ptr()),
                CredentialBlob: blob,
                CredentialBlobSize: (blob_length * 2) as u32,
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                ..Default::default()
            };

            CredWriteW(&credentials, 0)
        }
    }

    #[cfg(test)]
    pub(crate) mod tests {
        use super::*;

        #[test]
        fn test_store_blob() {
            let mut blob = vec![
                b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd',
            ];
            let target_name = "test_blob";
            store_blob_test(blob.as_mut_ptr(), blob.len(), target_name);
        }

        pub fn store_blob_test(blob: *mut u8, blob_length: usize, target_name: &str) {
            assert!(store_blob(blob, blob_length, target_name).is_ok())
        }
    }
}

#[cfg(unix)]
pub use unix::*;

#[cfg(unix)]
pub mod unix {
    use std::path::PathBuf;

    use super::*;
    use types::{AppPath, crypto::EncryptedMnemonic};
    use zeroize::Zeroize;

    pub fn store_db_encryption_salt(salt: Salt) -> Result<(), AppError> {
        let mut salt = salt.to_inner();

        store_blob(&salt, SALT_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to store salt, {}", err)))?;

        salt.zeroize();
        Ok(())
    }

    pub fn store_encrypted_mnemonic(
        encrypted_mnemonic: &EncryptedMnemonic,
    ) -> Result<(), AppError> {
        let mut blob = serde_json::to_vec(encrypted_mnemonic).map_err(|err| {
            AppError::Fatal(format!("Failed to parse Encrypted Mnemonic, {}", err))
        })?;

        store_blob(blob.as_slice(), ENCRYPTED_MNEMONIC_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to save EnctyptedMnemonic, {}", err)))?;

        blob.zeroize();

        Ok(())
    }

    fn store_blob(blob: &[u8], target_name: &str) -> Result<(), AppError> {
        let mut config_file = AppPath::get().config_directory();
        config_file.push(target_name);

        std::fs::write(config_file, blob)
            .map_err(|err| AppError::NonFatal(types::Notification::Warn(err.to_string())))
    }

    #[cfg(test)]
    pub(crate) mod tests {
        use std::env::{self, current_dir};

        use super::*;

        #[test]
        fn test_store_blob() {
            let blob = vec![
                b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd',
            ];
            let target_name = "test_blob";
            store_blob_test(blob.as_slice(), target_name);
        }

        pub fn store_blob_test(blob: &[u8], target_name: &str) {
            AppPath::get().create_directories_if_not_exists().ok();
            assert!(store_blob(blob, target_name).is_ok())
        }
    }
}
