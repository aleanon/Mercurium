use deps::zeroize::ZeroizeOnDrop;

mod account_repository;
mod fungible_asset_repository;
mod nft_asset_repository;
mod resource_repository;

pub use account_repository::AccountRepository;
pub use fungible_asset_repository::FungibleAssetRepository;
pub use nft_asset_repository::NftAssetRepository;
pub use resource_repository::ResourceRepository;

pub trait WalletDataRepository
where
    Self: Sized,
{
    type Key: ZeroizeOnDrop;
    type Path;
    type Error: std::error::Error;

    fn initialize(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    fn connect(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error>;

    fn delete(path: Self::Path, key: Self::Key) -> Result<(), Self::Error>;
}

/// Marker trait representing a complete wallet repository.
///
/// This trait is automatically implemented for any type that implements
/// all the required component repository traits, providing a convenient
/// single bound for generic functions that need full wallet functionality.
pub trait WalletDataRepo:
    AccountRepository + ResourceRepository + FungibleAssetRepository + NftAssetRepository
{
}

impl<T> WalletDataRepo for T where
    T: AccountRepository + ResourceRepository + FungibleAssetRepository + NftAssetRepository
{
}
