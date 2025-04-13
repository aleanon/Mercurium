use deps::*;

use store::DbError;
use thiserror::Error;
use tokio::task::JoinError;
use types::{crypto::{bip39, CryptoError, EncryptedMnemonicError}, AppPathError};

use super::task_runner::TaskError;


#[derive(Debug, Error)]
pub enum SetupError {
    #[error("Invalid seed phrase: {0}")]
    InvalidSeedPhrase(#[from] bip39::ErrorKind),
    #[error("Unable to generate key and salt: {0}")]
    UnableToGenerateKeyAndSalt(#[from] CryptoError),
    #[error("No password provided")]
    NoPasswordProvided,
    #[error("No mnemonic provided")]
    NoMnemonicProvided,
    #[error("Missing derived keys")]
    MissingDerivedKeys, 
    #[error("Application directory error: {0}")]
    AppPathError(#[from] AppPathError),
    #[error("Unable to update accounts")]
    UnableToUpdateAccounts,
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbError),
    #[error("Failed to encrypt mnemonic {0}")]
    EncryptedMnemonicError(#[from] EncryptedMnemonicError), 
    #[error("Failed to join task")]
    FailedToJoinTask,
    #[error("Tried to get result from a non started task")]
    AskedForValueOnATaskNotStarted,
    #[error("Attempted to get the value from a failed task")]
    FailedTask,
    #[error("Error while running task {0}")]
    TaskError(#[from] TaskError),
    #[error("Unspecified error during setup")]
    Unspecified,
}