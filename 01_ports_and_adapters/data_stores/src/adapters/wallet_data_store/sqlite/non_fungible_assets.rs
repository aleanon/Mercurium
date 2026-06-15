use deps::{
    async_sqlite::rusqlite::{self, Row, params},
    asynciter::{AsyncIterator, FromAsyncIterator, IntoAsyncIterator},
    futures::{self, StreamExt},
};

use crate::{
    adapters::data_store::sqlite::Sqlite,
    ports::wallet_data_store::{
        AccountId, AccountStore, AssetId, NftAssetStore, NonFungibleAsset, WalletDataStoreError,
    },
};

pub const CREATE_TABLE_NON_FUNGIBLE_ASSETS: &'static str = "CREATE TABLE IF NOT EXISTS
    non_fungible_assets (
        id BLOB NOT NULL PRIMARY KEY,
        resource_address BLOB NOT NULL,
        nfts BLOB NOT NULL,
        account_address BLOB NOT NULL,
        FOREIGN KEY(resource_address) REFERENCES resources(address),
        FOREIGN KEY(account_address) REFERENCES accounts(address)
    )
";
pub const UPSERT_NON_FUNGIBLE_ASSET: &'static str = "INSERT INTO
    non_fungible_assets (
        id,
        resource_address,
        nfts,
        account_address
    )
    VALUES (?, ?, ?, ?)
    ON CONFLICT (id)
    DO UPDATE SET
        nfts = excluded.nfts
";

#[async_trait::async_trait]
impl NftAssetStore for Sqlite {
    async fn upsert_non_fungible_assets_for_account<NonFungibleAssets>(
        &self,
        account_id: AccountId,
        assets: NonFungibleAssets,
    ) -> Result<(), WalletDataStoreError>
    where
        NonFungibleAssets: IntoIterator<Item = NonFungibleAsset> + Send + 'static,
    {
        self.transaction(UPSERT_NON_FUNGIBLE_ASSET, move |cached_stmt| {
            for non_fungible_asset in assets {
                cached_stmt.execute(params![
                    non_fungible_asset.id,
                    non_fungible_asset.resource_address,
                    non_fungible_asset.nfids,
                    account_id,
                ])?;
            }
            Ok(())
        })
        .await
        .map_err(Into::into)
    }

    async fn get_non_fungible_assets_for_account<T>(
        &self,
        account_id: AccountId,
    ) -> Option<(AccountId, T)>
    where
        T: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        self.query_map(
            "SELECT * FROM non_fungible_assets WHERE account_address = ?",
            [account_id.clone()],
            get_non_fungible_asset_from_row,
        )
        .await
        .ok()
        .map(|assets| (account_id, assets))
    }

    async fn get_all_non_fungible_assets_per_account<NonFungibleAssets, U>(
        &self,
    ) -> Result<NonFungibleAssets, WalletDataStoreError>
    where
        NonFungibleAssets:
            FromIterator<(AccountId, U)> + Send + 'static + Extend<(AccountId, U)> + Default,
        U: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        let account_addresses = self.get_account_addresses::<Vec<_>>().await?;

        let assets = futures::stream::iter(account_addresses)
            .filter_map(|account_address| self.get_non_fungible_assets_for_account(account_address))
            .collect()
            .await;

        Ok(assets)
    }

    async fn delete_non_fungible_asset(
        &self,
        asset_id: AssetId,
    ) -> Result<(), WalletDataStoreError> {
        todo!()
    }
}

fn get_non_fungible_asset_from_row(row: &Row<'_>) -> Result<NonFungibleAsset, rusqlite::Error> {
    Ok(NonFungibleAsset {
        id: row.get(0)?,
        resource_address: row.get(1)?,
        nfids: row.get(2)?,
    })
}
