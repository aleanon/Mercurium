mod account_repository;
mod fungible_asset_repository;
mod nft_asset_repository;
mod resource_repository;

use crate::services::repository::Repository;

pub use account_repository::AccountRepository;
pub use fungible_asset_repository::FungibleAssetRepository;
pub use nft_asset_repository::NftAssetRepository;
pub use resource_repository::ResourceRepository;

pub trait WalletRepository:
    AccountRepository + ResourceRepository + FungibleAssetRepository + NftAssetRepository
{
}

impl<T> WalletRepository for T where
    T: AccountRepository + ResourceRepository + FungibleAssetRepository + NftAssetRepository
{
}
