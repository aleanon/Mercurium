use std::collections::{BTreeMap, HashMap};

use crate::{statements, DbError, IconCache};

use super::{
    statements::{insert, upsert},
    AppDataDb,
};
use async_sqlite::rusqlite::params;
use types::{
    address::{AccountAddress, Address, ResourceAddress},
    assets::{FungibleAsset, NonFungibleAsset},
    crypto::HashedPassword,
    Account, Resource, Transaction, Us,
};

impl AppDataDb {
    pub async fn upsert_password_hash(&self, hash: HashedPassword) -> Result<(), DbError> {
        self.transaction(upsert::UPSERT_PASSWORD_HASH, move |cached_stmt| {
            cached_stmt.execute(params![1, hash])?;
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn upsert_account(&self, account: Account) -> Result<(), DbError> {
        self.transaction(upsert::UPSERT_ACCOUNT, move |cached_stmt| {
            cached_stmt.execute(params![
                account.address,
                account.id as i64,
                account.name,
                account.network,
                account.derivation_path,
                account.public_key.0,
                account.hidden,
                account.settings,
                account.balances_last_updated,
                account.transactions_last_updated,
            ])?;
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn upsert_accounts(&self, accounts: &[Account]) -> Result<(), DbError> {
        let accounts = unsafe { Us::new(accounts) };
        self.transaction(upsert::UPSERT_ACCOUNT, move |cached_stmt| {
            for account in accounts.iter() {
                cached_stmt.execute(params![
                    account.address,
                    account.id as i64,
                    account.name,
                    account.network,
                    account.derivation_path,
                    account.public_key.0,
                    account.hidden,
                    account.settings,
                    account.balances_last_updated,
                    account.transactions_last_updated,
                ])?;
            }
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn upsert_resources(&self, resources: Vec<Resource>) -> Result<(), DbError> {
        self.transaction(upsert::UPSERT_RESOURCE, move |cached_stmt| {
            for resource in resources.iter() {
                cached_stmt.execute(params![
                    resource.address,
                    resource.name,
                    resource.symbol,
                    resource.description,
                    resource.current_supply,
                    resource.divisibility,
                    resource.tags,
                ])?;
            }
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn upsert_fungible_assets_for_account(
        &self,
        account_address: AccountAddress,
        fungibles: Vec<FungibleAsset>,
    ) -> Result<(), DbError> {
        self.transaction(upsert::UPSERT_FUNGIBLE_ASSET, move |cached_stmt| {
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
        .await?;
        Ok(())
    }

    pub async fn upsert_non_fungible_assets_for_account(
        &self,
        account_address: AccountAddress,
        non_fungibles: &[NonFungibleAsset],
    ) -> Result<(), DbError> {
        let non_fungibles = unsafe { Us::new(non_fungibles) };

        self.transaction(upsert::UPSERT_NON_FUNGIBLE_ASSET, move |cached_stmt| {
            for non_fungible_asset in non_fungibles.iter() {
                cached_stmt.execute(params![
                    non_fungible_asset.id,
                    non_fungible_asset.resource_address,
                    non_fungible_asset.nfids,
                    account_address,
                ])?;
            }
            Ok(())
        })
        .await?;
        Ok(())
    }

    // pub async fn upsert_non_fungible_assets_for_account_v2<'a, T>(
    //     &self,
    //     account_address: AccountAddress,
    //     non_fungibles: T,
    // ) -> Result<(), DbError>
    // where
    //     T: Iterator<Item = &'a NonFungibleAsset>,
    // {
    //     let non_fungibles = unsafe { Ur::new(&non_fungibles) };

    //     self.transaction(upsert::UPSERT_NON_FUNGIBLE_ASSET, move |cached_stmt| {
    //         for non_fungible_asset in *non_fungibles {
    //             cached_stmt.execute(params![
    //                 non_fungible_asset.id,
    //                 non_fungible_asset.resource_address,
    //                 non_fungible_asset.nfids,
    //                 account_address,
    //             ])?;
    //         }

    //         Ok(())
    //     })
    //     .await?;
    //     Ok(())
    // }

    pub async fn update_transaction_status(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<(), DbError> {
        self.transaction(upsert::UPSERT_TRANSACTION, move |cached_stmt| {
            for transaction in transactions {
                cached_stmt.execute(params![
                    transaction.id,
                    transaction.transaction_address,
                    transaction.timestamp,
                    transaction.state_version as i64,
                ])?;
            }
            Ok(())
        })
        .await?;
        Ok(())
    }

    pub async fn insert_transactions(&self, transactions: Vec<Transaction>) -> Result<(), DbError> {
        self.conn_mut(|conn| {
            let tx = conn.transaction()?;

            {
                let mut transaction_stmt = tx.prepare_cached(upsert::UPSERT_TRANSACTION)?;
                let mut balance_changes_stmt = tx.prepare_cached(insert::INSERT_BALANCE_CHANGE)?;

                for transaction in transactions {
                    transaction_stmt.execute(params![
                        transaction.id,
                        transaction.transaction_address,
                        transaction.timestamp,
                        transaction.state_version as i64,
                    ])?;

                    for balance_change in &transaction.balance_changes {
                        balance_changes_stmt.execute(params![
                            balance_change.id,
                            balance_change.account,
                            balance_change.resource,
                            balance_change.nfids,
                            balance_change.amount,
                            transaction.transaction_address,
                        ])?;
                    }
                }
            }

            tx.commit().map_err(|err| err.into())
        })
        .await?;
        Ok(())
    }
}

impl IconCache {
    pub async fn upsert_resource_icons(
        &self,
        icons: HashMap<ResourceAddress, Vec<u8>>,
    ) -> Result<(), DbError> {
        self.conn_mut(move |conn| {
            let tx = conn.transaction()?;

            {
                let mut stmt = tx.prepare_cached(statements::upsert::UPSERT_RESOURCE_IMAGE)?;

                for (resource_address, image_data) in icons {
                    stmt.execute(params![resource_address, image_data])?;
                }
            }

            tx.commit()
        })
        .await?;
        Ok(())
    }

    pub async fn upsert_resource_icon(
        &self,
        resource_address: ResourceAddress,
        image_data: Vec<u8>,
    ) -> Result<(), DbError> {
        self.conn(move |conn| {
            conn.execute(
                statements::upsert::UPSERT_RESOURCE_IMAGE,
                params![resource_address, image_data],
            )
        })
        .await?;
        Ok(())
    }

    pub async fn upsert_nft_images(
        &self,
        resource_address: ResourceAddress,
        images: BTreeMap<String, Vec<u8>>,
    ) -> Result<(), DbError> {
        self.conn_mut(move |conn| {
            let tx = conn.transaction()?;

            {
                let mut stmt = tx.prepare_cached(statements::upsert::UPSERT_NFT_IMAGE)?;

                for (mut nfid, image_data) in images {
                    nfid.push_str(resource_address.checksum_as_str());
                    stmt.execute(params![nfid, image_data, resource_address])?;
                }
            }

            tx.commit()
        })
        .await?;
        Ok(())
    }

    pub async fn upsert_nft_image(
        &self,
        resource_address: ResourceAddress,
        mut nfid: String,
        image_data: Vec<u8>,
    ) -> Result<(), DbError> {
        self.conn(move |conn| {
            nfid.push_str(resource_address.as_str());
            conn.execute(
                statements::upsert::UPSERT_NFT_IMAGE,
                params![nfid, image_data, resource_address],
            )
        })
        .await?;
        Ok(())
    }
}
