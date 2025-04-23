use deps::*;

use super::statements::*;
use crate::DbError;
use async_sqlite::rusqlite::params;
use types::{
    address::AccountAddress, assets::{FungibleAsset, NonFungibleAsset}, crypto::HashedPassword, Account, Resource, Transaction
};

use super::AppDataDb;

impl AppDataDb {
    pub async fn upsert_password_hash(&self, hash: HashedPassword) -> Result<(), DbError> {
        self.transaction(password_hash::UPSERT_PASSWORD_HASH, move |cached_stmt| {
            cached_stmt.execute(params![1, hash])?;
            Ok(())
        })
        .await
    }

    pub async fn upsert_account(&self, account: Account) -> Result<(), DbError> {
        self.transaction(accounts::UPSERT_ACCOUNT, move |cached_stmt| {
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
        .await
    }

    pub async fn upsert_accounts<Accounts: IntoIterator<Item = Account> + Send + 'static>(&self, accounts: Accounts) -> Result<(), DbError> {
        self.transaction(accounts::UPSERT_ACCOUNT, move |cached_stmt| {
            for account in accounts {
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
        .await
    }

    pub async fn upsert_resources<Resources: IntoIterator<Item = Resource> + Send + 'static>(&self, resources: Resources) -> Result<(), DbError> {
        self.transaction(resources::UPSERT_RESOURCE, move |cached_stmt| {
            for resource in resources {
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
        .await
    }

    pub async fn upsert_fungible_assets_for_account<Fungibles: IntoIterator<Item = FungibleAsset> + Send + 'static>(
        &self,
        account_address: AccountAddress,
        fungibles: Fungibles,
    ) -> Result<(), DbError> {

        self.transaction(fungible_assets::UPSERT_FUNGIBLE_ASSET, move |cached_stmt| {
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
        .await
    }

    pub async fn upsert_non_fungible_assets_for_account<NonFungibles: IntoIterator<Item = NonFungibleAsset> + Send + 'static>(
        &self,
        account_address: AccountAddress,
        non_fungibles: NonFungibles,
    ) -> Result<(), DbError> {
        self.transaction(
            non_fungible_assets::UPSERT_NON_FUNGIBLE_ASSET,
            move |cached_stmt| {
                for non_fungible_asset in non_fungibles {
                    cached_stmt.execute(params![
                        non_fungible_asset.id,
                        non_fungible_asset.resource_address,
                        non_fungible_asset.nfids,
                        account_address,
                    ])?;
                }
                Ok(())
            },
        )
        .await
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
        self.transaction(transaction::UPSERT_TRANSACTION, move |cached_stmt| {
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
        .await
    }

    pub async fn insert_transactions(&self, transactions: Vec<Transaction>) -> Result<(), DbError> {
        self.conn_mut(|conn| {
            let tx = conn.transaction()?;

            {
                let mut transaction_stmt = tx.prepare_cached(transaction::UPSERT_TRANSACTION)?;
                let mut balance_changes_stmt =
                    tx.prepare_cached(balance_changes::INSERT_BALANCE_CHANGE)?;

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
        .await
    }
}
