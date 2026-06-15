use std::path::Path;

use async_trait::async_trait;
use thiserror::Error;
use types::crypto::{Key, KeyType};

#[derive(Debug, Error)]
pub enum DataStoreError {
    #[error("Wrong database key")]
    IncorrectKey,
    #[error("Unable to load database")]
    UnableToLoadRepository,
    #[error("Unable to create database")]
    UnableToCreateRepository,
    #[error("Unable to delete database")]
    UnableToDeleteRepository,
    #[error("Path error {0}")]
    PathError(#[from] app_path::Error),
    #[error("Repository error {0}")]
    RepositoryError(Box<dyn std::error::Error>),
}

#[async_trait]
pub trait DataStore: Sized + KeyType {
    async fn init_repository<P: AsRef<Path> + Send>(
        path: P,
        key: Key<Self>,
    ) -> Result<Self, DataStoreError>;

    async fn load<P: AsRef<Path> + Send>(path: P, key: Key<Self>) -> Result<Self, DataStoreError>;

    async fn delete_repository<P: AsRef<Path> + Send>(&self, path: P)
    -> Result<(), DataStoreError>;
}
