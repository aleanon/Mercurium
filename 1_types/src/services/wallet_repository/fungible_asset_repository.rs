use crate::services::repository::Repository;

pub trait FungibleAssetRepository: Repository {
    type AccountID;
    type FungibleAsset;
    type ID;

    fn upsert_fungible_asset(&self, asset: Self::FungibleAsset) -> Result<(), Self::Error>;

    fn updsert_fungible_assets<FungibleAssets: IntoIterator<Item = Self::FungibleAsset>>(
        &self,
        assets: FungibleAssets,
    ) -> Result<(), Self::Error>;

    fn get_fungible_asset(&self, asset_id: Self::ID) -> Result<Self::FungibleAsset, Self::Error>;

    fn get_all_fungible_assets_pr_account<FungibleAssets, T, U>(
        &self,
    ) -> Result<FungibleAssets, Self::Error>
    where
        FungibleAssets: FromIterator<(T, U)>,
        T: Into<Self::AccountID>,
        U: FromIterator<Self::FungibleAsset>;

    fn delete_fungible_asset(&self, asset_id: Self::ID) -> Result<(), Self::Error>;
}
