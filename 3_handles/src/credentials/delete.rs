use crate::credentials::{ENCRYPTED_MNEMONIC_TARGET_NAME, SALT_TARGET_NAME};
use types::AppError;
use zeroize::Zeroize;

#[cfg(windows)]
pub use mswindows::*;

#[cfg(windows)]
mod mswindows {
    use windows::{
        core::PCWSTR,
        Win32::Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC},
    };

    use super::*;

    pub fn delete_salt() -> Result<(), AppError> {
        delete_credentials(SALT_TARGET_NAME)
    }

    pub fn delete_encrypted_mnemonic() -> Result<(), AppError> {
        delete_credentials(ENCRYPTED_MNEMONIC_TARGET_NAME)
    }

    fn delete_credentials(target_name: &str) -> Result<(), AppError> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();
        target_name.push(0);

        let result: Result<(), AppError>;
        unsafe {
            match CredDeleteW(PCWSTR(target_name.as_mut_ptr()), CRED_TYPE_GENERIC, Some(0)) {
                Ok(_) => result = Ok(()),
                Err(err) => result = Err(AppError::Fatal(err.to_string())),
            }
        }

        target_name.zeroize();

        result
    }

    #[cfg(test)]
    pub(crate) mod tests {
        use crate::credentials::{
            get_credentials::tests::get_blob_test, store_credentials::tests::store_blob_test,
        };

        use super::*;

        #[test]
        fn test_delete_blob() {
            let mut blob = b"hello world".to_vec();
            let target_name = "test_blob";
            store_blob_test(blob.as_mut_ptr(), blob.len(), target_name);

            get_blob_test(target_name);

            delete_credentials_test(target_name)
        }

        pub fn delete_credentials_test(target_name: &str) {
            delete_credentials(target_name)
                .expect(format!("failed to delete credentials: {}", target_name).as_str());
        }
    }
}
