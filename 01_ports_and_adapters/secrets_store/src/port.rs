use types::{
    AppError,
    crypto::{EncryptedMnemonic, Salt},
};

/// Port for storing the wallet's secrets (the encrypted mnemonic and the database-encryption
/// salt) in the platform's secure store. Errors are surfaced as [`AppError`] to match the rest
/// of the wallet; the adapter ([`crate::OsCredentialStore`]) backs this with the OS credential
/// store (Windows) or the config directory (Unix).
pub trait SecretsStore {
    fn get_db_encryption_salt(&self) -> Result<Salt, AppError>;
    fn get_encrypted_mnemonic(&self) -> Result<EncryptedMnemonic, AppError>;

    fn store_db_encryption_salt(&self, salt: Salt) -> Result<(), AppError>;
    fn store_encrypted_mnemonic(&self, mnemonic: &EncryptedMnemonic) -> Result<(), AppError>;

    fn delete_db_encryption_salt(&self) -> Result<(), AppError>;
    fn delete_encrypted_mnemonic(&self) -> Result<(), AppError>;
}
