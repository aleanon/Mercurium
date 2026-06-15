use async_trait::async_trait;
use thiserror::Error;

use crate::ports::data_store::{self, DataStore};

type Icon = Vec<u8>;
type IconId = types::address::ResourceAddress;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Image not found")]
    ImageNotFound,
    #[error("Failed to save image")]
    FailedToSaveImage,
    #[error("Repository error {0}")]
    RepositoryError(#[from] data_store::DataStoreError),
}

#[async_trait]
pub trait IconDataStore: DataStore
where
    Self: Sized,
{
    async fn save_icon(&self, icon: Icon) -> Result<(), Error>;

    async fn save_icons(&self, icons: impl IntoIterator<Item = Icon>) -> Result<(), Error>;

    async fn get_icon(&self, id: IconId) -> Result<Icon, Error>;

    async fn get_icons(&self, ids: impl IntoIterator<Item = IconId>) -> Result<Vec<Icon>, Error>;

    async fn get_all_icons(&self) -> Result<impl FromIterator<Icon>, Error>;
}
