use deps::*;

use crate::credentials::{ENCRYPTED_MNEMONIC_TARGET_NAME, SALT_TARGET_NAME};
use types::AppError;

#[cfg(windows)]
pub use mswindows::*;

#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod mswindows {
    use windows::{
        Win32::Security::Credentials::{CRED_TYPE_GENERIC, CredDeleteW},
        core::PCWSTR,
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
            match CredDeleteW(PCWSTR(target_name.as_mut_ptr()), CRED_TYPE_GENERIC, None) {
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
            let mut blob = b"should be deleted".to_vec();
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

#[cfg(unix)]
mod unix {
    use super::*;
    use types::{AppPath, Notification};

    pub fn delete_salt() -> Result<(), AppError> {
        delete_credentials(SALT_TARGET_NAME)
    }

    pub fn delete_encrypted_mnemonic() -> Result<(), AppError> {
        delete_credentials(ENCRYPTED_MNEMONIC_TARGET_NAME)
    }

    fn delete_credentials(target_name: &str) -> Result<(), AppError> {
        let mut config_file = AppPath::get().config_directory();
        config_file.push(target_name);

        std::fs::remove_file(config_file)
            .map_err(|err| AppError::NonFatal(Notification::Warn(err.to_string())))?;

        Ok(())
    }

    #[cfg(test)]
    pub(crate) mod tests {
        use crate::credentials::{
            get_credentials::tests::get_blob_test, store_credentials::tests::store_blob_test,
        };

        use super::*;

        #[test]
        fn test_delete_blob() {
            let blob = b"hello world should be deleted";
            let target_name = "test_blob";
            store_blob_test(blob, target_name);

            get_blob_test(target_name);

            delete_credentials_test(target_name)
        }

        pub fn delete_credentials_test(target_name: &str) {
            delete_credentials(target_name)
                .expect(format!("failed to delete credentials: {}", target_name).as_str());
        }
    }
}
