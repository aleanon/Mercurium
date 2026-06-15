use std::path::Path;

use async_trait::async_trait;
use thiserror::Error;
use types::crypto::Key;

use crate::ports::data_store::{self, DataStore};

#[derive(Debug, Error)]
pub enum WalletDataStoreError {
    #[error("Account not found")]
    AccountNotFound,
    #[error("Resource not found")]
    ResourceNotFound,
    #[error("Fungible asset not found")]
    FungibleAssetNotFound,
    #[error("NFT asset not found")]
    NftAssetNotFound,
    #[error("Repository error")]
    RepositoryError(#[from] data_store::DataStoreError),
}

pub type Account = types::Account;
pub type Resource = types::Resource;
pub type AccountId = types::address::AccountAddress;
pub type ResourceId = types::address::ResourceAddress;
pub type FungibleAsset = types::assets::FungibleAsset;
pub type NonFungibleAsset = types::assets::NonFungibleAsset;
pub type AssetId = types::assets::AssetId;

#[async_trait]
pub trait WalletDataStore:
    DataStore + AccountStore + ResourceStore + FungibleAssetStore + NftAssetStore
{
    async fn init_repository<P: AsRef<Path> + Send>(
        path: P,
        key: Key<Self>,
    ) -> Result<Self, WalletDataStoreError>;

    async fn load<P: AsRef<Path> + Send>(
        path: P,
        key: Key<Self>,
    ) -> Result<Self, WalletDataStoreError>;

    async fn delete_repository<P: AsRef<Path> + Send>(
        &self,
        path: P,
    ) -> Result<(), WalletDataStoreError>;
}

#[async_trait]
pub trait AccountStore: DataStore {
    async fn upsert_account(&mut self, account: Account) -> Result<(), WalletDataStoreError>;

    async fn upsert_accounts<Accounts>(
        &mut self,
        accounts: Accounts,
    ) -> Result<(), WalletDataStoreError>
    where
        Accounts: IntoIterator<Item = Account> + Send + 'static;

    async fn get_account(&self, account_id: AccountId) -> Result<Account, WalletDataStoreError>;

    async fn get_account_addresses<T>(&self) -> Result<T, WalletDataStoreError>
    where
        T: FromIterator<AccountId> + Send + 'static;

    async fn get_all_accounts<Accounts>(&self) -> Result<Accounts, WalletDataStoreError>
    where
        Accounts: FromIterator<Account> + Send + 'static;

    async fn delete_account(&self, account_id: AccountId) -> Result<(), WalletDataStoreError>;
}

#[async_trait]
pub trait FungibleAssetStore: DataStore {
    async fn upsert_fungible_assets_for_account<
        Fungibles: IntoIterator<Item = FungibleAsset> + Send + 'static,
    >(
        &self,
        account_address: AccountId,
        fungibles: Fungibles,
    ) -> Result<(), WalletDataStoreError>;

    async fn get_fungible_assets_for_account<T>(
        &self,
        account_address: AccountId,
    ) -> Option<(AccountId, T)>
    where
        T: FromIterator<FungibleAsset> + Send + 'static;

    async fn get_fungible_assets_for_accounts<T, U>(
        &self,
        account_addresses: Vec<AccountId>,
    ) -> Result<T, WalletDataStoreError>
    where
        T: FromIterator<(AccountId, U)> + Send + 'static + Extend<(AccountId, U)> + Default,
        U: FromIterator<FungibleAsset> + Send + 'static;

    async fn get_all_fungible_assets_pr_account<FungibleAssets, T>(
        &self,
    ) -> Result<FungibleAssets, WalletDataStoreError>
    where
        FungibleAssets: FromIterator<(AccountId, T)> + Send + 'static,
        T: FromIterator<FungibleAsset>;

    async fn delete_fungible_asset(&self, asset_id: AssetId) -> Result<(), WalletDataStoreError>;
}

#[async_trait]
pub trait NftAssetStore: DataStore + AccountStore {
    async fn upsert_non_fungible_assets_for_account<NonFungibleAssets>(
        &self,
        account_id: AccountId,
        assets: NonFungibleAssets,
    ) -> Result<(), WalletDataStoreError>
    where
        NonFungibleAssets: IntoIterator<Item = NonFungibleAsset> + Send + 'static;

    async fn get_non_fungible_assets_for_account<T>(
        &self,
        account_id: AccountId,
    ) -> Option<(AccountId, T)>
    where
        T: FromIterator<NonFungibleAsset> + Send + 'static;

    async fn get_all_non_fungible_assets_per_account<NonFungibleAssets, U>(
        &self,
    ) -> Result<NonFungibleAssets, WalletDataStoreError>
    where
        NonFungibleAssets:
            FromIterator<(AccountId, U)> + Send + 'static + Extend<(AccountId, U)> + Default,
        U: FromIterator<NonFungibleAsset> + Send + 'static;

    async fn delete_non_fungible_asset(
        &self,
        asset_id: AssetId,
    ) -> Result<(), WalletDataStoreError>;
}

#[async_trait]
pub trait ResourceStore: DataStore {
    async fn upsert_resource(&self, resource: Resource) -> Result<Resource, WalletDataStoreError>;

    async fn upsert_resources<Resources>(
        &self,
        resources: Resources,
    ) -> Result<(), WalletDataStoreError>
    where
        Resources: IntoIterator<Item = Resource> + Send + 'static;

    async fn get_resource(&self, resource_id: ResourceId)
    -> Result<Resource, WalletDataStoreError>;

    async fn get_all_resources<Resources>(&self) -> Result<Resources, WalletDataStoreError>
    where
        Resources: FromIterator<Resource> + Send + 'static;

    async fn delete_resource(&self, resource_id: ResourceId) -> Result<(), WalletDataStoreError>;
}
