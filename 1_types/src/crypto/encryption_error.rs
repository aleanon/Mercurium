use thiserror::Error;



#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Failed to create random value")]
    FailedToCreateRandomValue,
}