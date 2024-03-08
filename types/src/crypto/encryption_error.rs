use thiserror::Error;



#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Failed to create random value")]
    FailedToCreateRandomValue,
}