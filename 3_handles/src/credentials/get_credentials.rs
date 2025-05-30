use deps::*;

use crate::credentials::{ENCRYPTED_MNEMONIC_TARGET_NAME, SALT_TARGET_NAME};
use types::{AppError, crypto::Salt};

#[cfg(windows)]
pub use mswindows::*;

#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod mswindows {
    use super::*;
    use types::crypto::EncryptedMnemonic;
    use windows::{
        Win32::{
            Foundation::E_POINTER,
            Security::Credentials::{CRED_TYPE_GENERIC, CREDENTIALW, CredFree, CredReadW},
        },
        core::PCWSTR,
    };

    pub fn get_db_encryption_salt() -> Result<Salt, AppError> {
        get_blob(SALT_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to get credentials blob: {err}")))
            .and_then(|blob| {
                blob.try_into()
                    .map_err(|err| AppError::Fatal(format!("Failed to get Salt: {err}")))
            })
    }

    pub fn get_encrypted_mnemonic() -> Result<EncryptedMnemonic, AppError> {
        get_blob(ENCRYPTED_MNEMONIC_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to get credentials blob: {err}")))
            .and_then(|blob| {
                serde_json::from_slice(&blob).map_err(|err| {
                    AppError::Fatal(format!("Failed to parse blob to Encrypted Mnemonic: {err}"))
                })
            })
    }

    fn get_blob(target_name: &str) -> windows::core::Result<Vec<u8>> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();
        target_name.push(0);

        let mut cred_ptr: *mut CREDENTIALW = std::ptr::null_mut();

        unsafe {
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

                    CredFree(cred_ptr as *mut _);
                    Ok(slice.to_vec())
                } else {
                    Err(windows::core::Error::new(
                        E_POINTER,
                        "Null pointer received for credentials",
                    ))
                }
            })
        }
    }

    #[cfg(test)]
    pub(crate) mod tests {
        use super::*;
        use crate::credentials::store_credentials::tests::store_blob_test;

        #[test]
        fn test_get_blob() {
            let mut blob = vec![
                b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd',
            ];
            let target_name = "test_blob";
            store_blob_test(blob.as_mut_ptr(), blob.len(), target_name);

            get_blob_test(target_name);
        }

        pub fn get_blob_test(target_name: &str) -> Vec<u8> {
            get_blob(target_name).expect("Failed to get blob")
        }
    }
}

#[cfg(unix)]
mod unix {
    use std::io::Read;

    use super::*;
    use types::{AppPath, crypto::EncryptedMnemonic};

    pub fn get_db_encryption_salt() -> Result<Salt, AppError> {
        get_blob(SALT_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to get credentials blob: {err}")))
            .and_then(|blob| {
                blob.try_into()
                    .map_err(|err| AppError::Fatal(format!("Failed to get Salt: {err}")))
            })
    }

    pub fn get_encrypted_mnemonic() -> Result<EncryptedMnemonic, AppError> {
        get_blob(ENCRYPTED_MNEMONIC_TARGET_NAME)
            .map_err(|err| AppError::Fatal(format!("Failed to get credentials blob: {err}")))
            .and_then(|blob| {
                serde_json::from_slice(&blob).map_err(|err| {
                    AppError::Fatal(format!("Failed to parse blob to Encrypted Mnemonic: {err}"))
                })
            })
    }

    fn get_blob(target_name: &str) -> Result<Vec<u8>, AppError> {
        let mut config_file = AppPath::get().config_directory();
        config_file.push(target_name);

        std::fs::read(config_file)
            .map_err(|err| AppError::NonFatal(types::Notification::Warn(err.to_string())))
    }

    #[cfg(test)]
    pub(crate) mod tests {
        use super::*;
        use crate::credentials::store_credentials::tests::store_blob_test;

        #[test]
        fn test_get_blob() {
            let blob = vec![
                b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd',
            ];

            let target_name = "test_blob";
            store_blob_test(blob.as_slice(), target_name);

            get_blob_test(target_name);
        }

        pub fn get_blob_test(target_name: &str) -> Vec<u8> {
            get_blob(target_name).expect("Failed to get blob")
        }
    }
}
