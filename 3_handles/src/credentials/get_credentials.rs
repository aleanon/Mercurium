use deps::*;

use crate::credentials::{ENCRYPTED_MNEMONIC_TARGET_NAME, SALT_TARGET_NAME};
use types::{crypto::Salt, AppError};

#[cfg(windows)]
pub use mswindows::*;

#[cfg(windows)]
mod mswindows {
    use super::*;
    use types::crypto::EncryptedMnemonic;
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::E_POINTER,
            Security::Credentials::{CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC},
        },
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
                0,
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
        use crate::credentials::store_credentials::tests::store_blob_test;
        use super::*;

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
