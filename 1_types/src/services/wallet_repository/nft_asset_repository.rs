use crate::services::repository::Repository;

pub trait NftAssetRepository: Repository {
    type AccountId;
    type NonFungibleAsset;
    type ID;

    fn upsert_non_fungible_asset(&self, asset: Self::NonFungibleAsset) -> Result<(), Self::Error>;

    fn upsert_non_fungible_assets<NonFungibleAssets: IntoIterator<Item = Self::NonFungibleAsset>>(
        &self,
        assets: NonFungibleAssets,
    ) -> Result<(), Self::Error>;

    fn get_non_fungible_asset(
        &self,
        asset_id: Self::ID,
    ) -> Result<Self::NonFungibleAsset, Self::Error>;

    fn get_all_non_fungible_assets_per_account<NonFungibleAssets, T, U>(
        &self,
    ) -> Result<Vec<Self::NonFungibleAsset>, Self::Error>
    where
        NonFungibleAssets: FromIterator<(T, U)>,
        T: Into<Self::AccountId>,
        U: FromIterator<Self::NonFungibleAsset>;

    fn delete_non_fungible_asset(&self, asset_id: Self::ID) -> Result<(), Self::Error>;
}
