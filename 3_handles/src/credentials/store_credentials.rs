use deps::*;

use super::ENCRYPTED_MNEMONIC_TARGET_NAME;
use super::SALT_TARGET_NAME;
use types::{crypto::Salt, AppError};

#[cfg(windows)]
pub use mswindows::*;

#[cfg(windows)]
pub mod mswindows {
    use super::*;
    use types::crypto::EncryptedMnemonic;
    use windows::{
        core::PWSTR,
        Win32::Security::Credentials::{
            CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
        },
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
