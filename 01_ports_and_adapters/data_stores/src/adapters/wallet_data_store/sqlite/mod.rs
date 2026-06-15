mod accounts;
mod fungible_assets;
mod non_fungible_assets;
mod resources;

use std::path::Path;

use async_trait::async_trait;
use deps::const_format;
use types::crypto::Key;

use crate::{
    adapters::data_store::sqlite::Sqlite,
    ports::{
        data_store::DataStore,
        wallet_data_store::{WalletDataStore, WalletDataStoreError},
    },
};

use accounts::CREATE_TABLE_ACCOUNTS;
use fungible_assets::CREATE_TABLE_FUNGIBLE_ASSETS;
use non_fungible_assets::CREATE_TABLE_NON_FUNGIBLE_ASSETS;
use resources::CREATE_TABLE_RESOURCES;

pub const CREATE_ALL_TABLES: &'static str = const_format::formatcp!(
    "BEGIN;
    {CREATE_TABLE_ACCOUNTS};
    {CREATE_TABLE_RESOURCES};
    {CREATE_TABLE_FUNGIBLE_ASSETS};
    {CREATE_TABLE_NON_FUNGIBLE_ASSETS};
    COMMIT;"
);

pub const DROP_TABLES: &'static str = const_format::formatcp!(
    "BEGIN;
    DROP TABLE accounts;
    DROP TABLE resources;
    DROP TABLE fungible_assets;
    DROP TABLE non_fungible_assets;
    COMMIT;"
);

#[async_trait]
impl WalletDataStore for Sqlite
where
    Sqlite: DataStore,
{
    async fn init_repository<P: AsRef<Path> + Send>(
        path: P,
        key: Key<Self>,
    ) -> Result<Self, WalletDataStoreError> {
        let repository = <Self as DataStore>::init_repository(path, key).await?;
        repository.execute_batch(CREATE_ALL_TABLES).await?;
        Ok(repository)
    }

    async fn load<P: AsRef<Path> + Send>(
        path: P,
        key: Key<Self>,
    ) -> Result<Self, WalletDataStoreError> {
        Ok(<Self as DataStore>::load(path, key).await?)
    }

    async fn delete_repository<P: AsRef<Path> + Send>(
        &self,
        path: P,
    ) -> Result<(), WalletDataStoreError> {
        self.execute_batch(DROP_TABLES).await?;
        <Self as DataStore>::delete_repository(self, path).await?;
        Ok(())
    }
}
