use deps::{
    async_sqlite::rusqlite::{self, Row, params},
    futures::{self, StreamExt},
};

use crate::{
    adapters::data_store::sqlite::Sqlite,
    ports::wallet_data_store::{
        AccountId, AssetId, FungibleAsset, FungibleAssetStore, WalletDataStoreError,
    },
};

pub const CREATE_TABLE_FUNGIBLE_ASSETS: &'static str = "CREATE TABLE IF NOT EXISTS
    fungible_assets (
        id BLOB NOT NULL PRIMARY KEY,
        resource_address BLOB NOT NULL,
        amount TEXT NOT NULL,
        account_address BLOB NOT NULL,
        FOREIGN KEY(resource_address) REFERENCES resources(address),
        FOREIGN KEY(account_address) REFERENCES accounts(address)
    )
";

pub const UPSERT_FUNGIBLE_ASSET: &'static str = "INSERT INTO
    fungible_assets (
        id,
        resource_address,
        amount,
        account_address
    )
    VALUES (?, ?, ?, ?)
    ON CONFLICT (id)
    DO UPDATE SET
        amount = excluded.amount
";

#[async_trait::async_trait]
impl FungibleAssetStore for Sqlite {
    async fn upsert_fungible_assets_for_account<
        Fungibles: IntoIterator<Item = FungibleAsset> + Send + 'static,
    >(
        &self,
        account_address: AccountId,
        fungibles: Fungibles,
    ) -> Result<(), WalletDataStoreError> {
        Ok(self
            .transaction(UPSERT_FUNGIBLE_ASSET, move |cached_stmt| {
                for fungible_asset in fungibles {
                    cached_stmt.execute(params![
                        fungible_asset.id,
                        fungible_asset.resource_address,
                        fungible_asset.amount,
                        account_address,
                    ])?;
                }
                Ok(())
            })
            .await?)
    }

    async fn get_fungible_assets_for_account<T>(
        &self,
        account_address: AccountId,
    ) -> Option<(AccountId, T)>
    where
        T: FromIterator<FungibleAsset> + Send + 'static,
    {
        self.query_map(
            "SELECT * FROM fungible_assets WHERE account_address = ?",
            [account_address.clone()],
            get_fungible_asset_from_row,
        )
        .await
        .ok()
        .map(|assets| (account_address, assets))
    }

    async fn get_fungible_assets_for_accounts<T, U>(
        &self,
        account_addresses: Vec<AccountId>,
    ) -> Result<T, WalletDataStoreError>
    where
        T: FromIterator<(AccountId, U)> + Send + 'static + Extend<(AccountId, U)> + Default,
        U: FromIterator<FungibleAsset> + Send + 'static,
    {
        let assets = futures::stream::iter(account_addresses)
            .filter_map(|account_address| self.get_fungible_assets_for_account(account_address))
            .collect()
            .await;

        Ok(assets)
    }

    async fn get_all_fungible_assets_pr_account<FungibleAssets, T>(
        &self,
    ) -> Result<FungibleAssets, WalletDataStoreError>
    where
        FungibleAssets: FromIterator<(AccountId, T)> + Send + 'static,
        T: FromIterator<FungibleAsset>,
    {
        todo!()
    }

    async fn delete_fungible_asset(&self, _asset_id: AssetId) -> Result<(), WalletDataStoreError> {
        todo!()
    }
}

fn get_fungible_asset_from_row(row: &Row<'_>) -> Result<FungibleAsset, rusqlite::Error> {
    Ok(FungibleAsset {
        id: row.get(0)?,
        resource_address: row.get(1)?,
        amount: row.get(2)?,
    })
}
