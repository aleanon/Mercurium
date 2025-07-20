use crate::wallet_data_repository::WalletDataRepository;

pub trait FungibleAssetRepository: WalletDataRepository {
    type AccountID;
    type FungibleAsset;
    type FungibleAssetID;

    fn upsert_fungible_asset(&self, asset: Self::FungibleAsset) -> Result<(), Self::Error>;

    fn updsert_fungible_assets<FungibleAssets: IntoIterator<Item = Self::FungibleAsset>>(
        &self,
        assets: FungibleAssets,
    ) -> Result<(), Self::Error>;

    fn get_fungible_asset(
        &self,
        asset_id: Self::FungibleAssetID,
    ) -> Result<Self::FungibleAsset, Self::Error>;

    fn get_all_fungible_assets_pr_account<FungibleAssets, T, U>(
        &self,
    ) -> Result<FungibleAssets, Self::Error>
    where
        FungibleAssets: FromIterator<(T, U)>,
        T: Into<Self::AccountID>,
        U: FromIterator<Self::FungibleAsset>;

    fn delete_fungible_asset(&self, asset_id: Self::FungibleAssetID) -> Result<(), Self::Error>;
}
