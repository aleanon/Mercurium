use crate::app_path;
use async_trait::async_trait;
use deps::zeroize::ZeroizeOnDrop;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Wrong database key")]
    IncorrectKey,
    #[error("Unable to load database")]
    UnableToLoadDatabase,
    #[error("Unable to create database")]
    UnableToCreateDatabase,
    #[error("Path error {0}")]
    PathError(#[from] app_path::Error),
}

/// Marker trait representing a complete wallet repository.
///
/// This trait is automatically implemented for any type that implements
/// all the required component repository traits, providing a convenient
/// single bound for generic functions that need full wallet data repository functionality.
pub trait WalletDataRepo:
    AccountRepository + ResourceRepository + FungibleAssetRepository + NftAssetRepository
{
}

impl<T> WalletDataRepo for T where
    T: WalletDataRepository
        + AccountRepository
        + ResourceRepository
        + FungibleAssetRepository
        + NftAssetRepository
{
}

#[async_trait]
pub trait WalletDataRepository: Sized {
    type Key: ZeroizeOnDrop;
    type Path;
    type WalletData;

    async fn init_repository(path: Self::Path, key: Self::Key) -> Result<Self, Error>;

    async fn load(path: Self::Path, key: Self::Key) -> Result<Self, Error>;

    async fn get_all_wallet_data(&self) -> Result<Self::WalletData, Error>;

    async fn delete_repository(&self, key: Self::Key) -> Result<(), Error>;
}

#[async_trait]
pub trait AccountRepository {
    type Account;
    type AccountId;

    async fn upsert_account(&mut self, account: Self::Account) -> Result<(), Error>;

    async fn upsert_accounts<Accounts: IntoIterator<Item = Self::Account>>(
        &mut self,
        accounts: Accounts,
    ) -> Result<(), Error>;

    async fn get_account(&self, account_id: Self::AccountId) -> Result<Self::Account, Error>;

    async fn get_all_accounts<Accounts: FromIterator<Self::Account>>(
        &self,
    ) -> Result<Accounts, Error>;

    async fn delete_account(&self, account_id: Self::AccountId) -> Result<(), Error>;
}
#[async_trait]
pub trait FungibleAssetRepository {
    type AccountID;
    type FungibleAsset;
    type FungibleAssetID;

    async fn upsert_fungible_asset(&self, asset: Self::FungibleAsset) -> Result<(), Error>;

    async fn updsert_fungible_assets<FungibleAssets: IntoIterator<Item = Self::FungibleAsset>>(
        &self,
        assets: FungibleAssets,
    ) -> Result<(), Error>;

    async fn get_fungible_asset(
        &self,
        asset_id: Self::FungibleAssetID,
    ) -> Result<Self::FungibleAsset, Error>;

    async fn get_all_fungible_assets_pr_account<FungibleAssets, T, U>(
        &self,
    ) -> Result<FungibleAssets, Error>
    where
        FungibleAssets: FromIterator<(T, U)>,
        T: Into<Self::AccountID>,
        U: FromIterator<Self::FungibleAsset>;

    async fn delete_fungible_asset(&self, asset_id: Self::FungibleAssetID) -> Result<(), Error>;
}

#[async_trait]
pub trait NftAssetRepository {
    type AccountId;
    type NonFungibleAsset;
    type NonFungibleAssetID;

    async fn upsert_non_fungible_asset(&self, asset: Self::NonFungibleAsset) -> Result<(), Error>;

    async fn upsert_non_fungible_assets<
        NonFungibleAssets: IntoIterator<Item = Self::NonFungibleAsset>,
    >(
        &self,
        assets: NonFungibleAssets,
    ) -> Result<(), Error>;

    async fn get_non_fungible_asset(
        &self,
        asset_id: Self::NonFungibleAssetID,
    ) -> Result<Self::NonFungibleAsset, Error>;

    async fn get_all_non_fungible_assets_per_account<NonFungibleAssets, T, U>(
        &self,
    ) -> Result<Vec<Self::NonFungibleAsset>, Error>
    where
        NonFungibleAssets: FromIterator<(T, U)>,
        T: Into<Self::AccountId>,
        U: FromIterator<Self::NonFungibleAsset>;

    async fn delete_non_fungible_asset(
        &self,
        asset_id: Self::NonFungibleAssetID,
    ) -> Result<(), Error>;
}

#[async_trait]
pub trait ResourceRepository {
    type Resource;
    type ResourceID;

    async fn upsert_resource(&self, resource: Self::Resource) -> Result<Self::Resource, Error>;

    async fn upsert_resources<Resources: IntoIterator<Item = Self::Resource>>(
        &self,
        resources: Resources,
    ) -> Result<(), Error>;

    async fn get_resource(&self, resource_id: Self::ResourceID) -> Result<Self::Resource, Error>;

    async fn get_all_resources<Resources: FromIterator<Self::Resource>>(
        &self,
    ) -> Result<Resources, Error>;

    async fn delete_resource(&self, resource_id: Self::ResourceID) -> Result<(), Error>;
}
