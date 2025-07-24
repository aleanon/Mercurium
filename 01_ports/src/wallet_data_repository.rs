use deps::zeroize::ZeroizeOnDrop;
use thiserror::Error;

mod account_repository;
mod fungible_asset_repository;
mod nft_asset_repository;
mod resource_repository;

pub use account_repository::AccountRepository;
pub use fungible_asset_repository::FungibleAssetRepository;
pub use nft_asset_repository::NftAssetRepository;
pub use resource_repository::ResourceRepository;

use crate::app_path;

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
    T: AccountRepository + ResourceRepository + FungibleAssetRepository + NftAssetRepository
{
}

pub trait WalletDataRepository
where
    Self: Sized,
{
    type Key: ZeroizeOnDrop;
    type Path;
    type WalletData;
    type Error: Into<Error>;

    fn init_repository(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    fn get_all_wallet_data(&self) -> Result<Self::WalletData, Self::Error>;

    fn connect(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    fn delete_repository(path: Self::Path, key: Self::Key) -> Result<(), Self::Error>;
}

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
