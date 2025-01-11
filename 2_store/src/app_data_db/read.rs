use std::collections::{BTreeSet, HashMap};

use async_sqlite::rusqlite::{self, Row};
use asynciter::{AsyncIterator, FromAsyncIterator, IntoAsyncIterator};
use types::{
    address::AccountAddress,
    assets::{FungibleAsset, NonFungibleAsset},
    crypto::HashedPassword,
    Account, BalanceChange, Ed25519PublicKey, Resource, Transaction, TransactionId,
};

use crate::DbError;

use super::AppDataDb;

impl AppDataDb {
    pub async fn get_db_password_hash(&self) -> Result<HashedPassword, DbError> {
        self.query_row(
            "SELECT password FROM password_hash WHERE id = 1",
            [],
            |row| Ok(row.get(0)?),
        )
        .await
        .map_err(|err| err.into())
    }

    pub async fn get_account(&self, account_address: AccountAddress) -> Result<Account, DbError> {
        self.query_row(
            "SELECT * FROM accounts WHERE address = ?",
            [account_address],
            Self::get_account_from_row,
        )
        .await
        .map_err(|err| err.into())
    }

    

    pub async fn get_account_addresses<T>(&self) -> Result<T, DbError>
    where
        T: FromIterator<AccountAddress> + Send + 'static,
    {
        self.query_map(
            "SELECT address FROM accounts",
            [], 
            |row| Ok(row.get(0)?))
        .await
    }

    pub async fn get_accounts<T>(&self) -> Result<T, DbError>
    where
        T: FromIterator<Account> + Send + 'static,
    {
        self.query_map(
            "SELECT * FROM accounts", 
            [], 
            Self::get_account_from_row
        ).await
    }

    pub async fn get_all_fungible_assets_per_account<T, U>(&self) -> Result<T, DbError>
    where
        T: FromAsyncIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<FungibleAsset> + Send + 'static,
    {
        let accounts = self.get_account_addresses::<Vec<_>>().await?;

        let all_fungibles = accounts
            .into_aiter()
            .afilter_map(|address| self.get_fungible_assets_for_account(address))
            .collect()
            .await;

        Ok(all_fungibles)
    }

    pub async fn get_fungible_assets_for_account<T>(
        &self,
        account_address: AccountAddress,
    ) -> Option<(AccountAddress, T)>
    where
        T: FromIterator<FungibleAsset> + Send + 'static,
    {
        self.query_map(
            "SELECT * FROM fungible_assets WHERE account_address = ?",
            [account_address.clone()],
            Self::get_fungible_asset_from_row,
        )
        .await
        .ok()
        .map(|assets| (account_address, assets))
    }

    pub async fn get_fungible_assets_for_accounts<T, U>(
        &self,
        account_addresses: Vec<AccountAddress>,
    ) -> Result<T, DbError>
    where
        T: FromAsyncIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<FungibleAsset> + Send + 'static,
    {
        let all_fungibles = account_addresses
            .into_aiter()
            .afilter_map(|account_address| 
                self.get_fungible_assets_for_account(account_address))
            .collect()
            .await;

        Ok(all_fungibles)
    }

    pub async fn get_all_non_fungible_assets_per_account<T, U>(&self) -> Result<T, DbError>
    where
        T: FromAsyncIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        let account_addresses = self.get_account_addresses::<Vec<_>>().await?;

        let all_fungibles = account_addresses
            .into_aiter()
            .afilter_map(|account_address| {
                self.get_non_fungible_assets_for_account(account_address)
            })
            .collect()
            .await;

        Ok(all_fungibles)
    }

    async fn get_non_fungible_assets_for_account<T>(
        &self,
        account_address: AccountAddress,
    ) -> Option<(AccountAddress, T)>
    where
        T: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        self.query_map(
            "SELECT * FROM non_fungible_assets WHERE account_address = ?",
            [account_address.clone()],
            Self::get_non_fungible_asset_from_row,
        )
        .await
        .ok()
        .map(|assets| (account_address, assets))
    }

    pub async fn get_non_fungible_assets_for_accounts<T, U>(
        &self,
        account_addresses: Vec<AccountAddress>,
    ) -> Result<T, DbError>
    where
        T: FromAsyncIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        let all_fungibles = account_addresses
            .into_aiter()
            .afilter_map(|account_address| {
                self.get_non_fungible_assets_for_account(account_address)
            })
            .collect()
            .await;

        Ok(all_fungibles)
    }

    pub async fn get_all_resources<T>(&self) -> Result<T, DbError>
    where
        T: FromIterator<Resource> + Send + 'static,
    {
        self.query_map(
            "SELECT * FROM resources", 
            [], 
            Self::get_resource_from_row
        )
        .await
    }

    pub async fn get_last_transaction_for_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<Transaction, DbError> {
        let mut transactions: BTreeSet<Transaction> =
            self.get_transactions_for_account(account_address).await?;

        transactions
            .pop_last()
            .ok_or(async_sqlite::Error::Rusqlite(rusqlite::Error::QueryReturnedNoRows).into())
    }

    pub async fn get_transactions_for_account<T>(
        &self,
        account_address: AccountAddress,
    ) -> Result<T, DbError>
    where
        T: FromAsyncIterator<Transaction> + Send + 'static,
    {
        let balance_changes = self
            .get_balance_changes_for_account(account_address)
            .await?;

        let transactions: T = balance_changes
            .into_aiter()
            .afilter_map(|(transaction_id, balance_changes)| {
                self.get_transaction(transaction_id, balance_changes)
            })
            .collect()
            .await;

        Ok(transactions)
    }

    async fn get_transaction(
        &self,
        transaction_id: TransactionId,
        balance_changes: Vec<BalanceChange>,
    ) -> Option<Transaction> {
        self.query_row(
            "SELECT * FROM transactions WHERE transaction_id = ?",
            [transaction_id],
            |row| Self::get_transaction_from_row(row, balance_changes),
        )
        .await
        .ok()
    }

    pub async fn get_balance_changes_for_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<HashMap<TransactionId, Vec<BalanceChange>>, DbError> {
        self.prepare_cached_statement(
            "SELECT * FROM balance_changes WHERE account_address = ?",
            |cached_stmt| {
                let balance_changes = cached_stmt
                    .query_map(
                        [account_address],
                        Self::get_transactionid_and_balance_change_from_row,
                    )?
                    .filter_map(|result| result.ok())
                    .fold(HashMap::new(), Self::fold_transactions_and_balance_changes);
                Ok(balance_changes)
            },
        )
        .await
    } 

    pub async fn get_all_transactions<T>(&self) -> Result<T, DbError>
    where
        T: FromAsyncIterator<Transaction> + Send + 'static,
    {
        let transactions: Vec<Transaction> = self
            .query_map(
                "SELECT * FROM transactions",
                 [], 
                 |row| Self::get_transaction_from_row(row, Vec::new()))
            .await?;

        let transactions = transactions
            .into_aiter()
            .amap(|mut transaction| async {
                transaction.balance_changes = self
                    .get_balance_changes_for_transaction(transaction.id.clone())
                    .await;
                transaction
            })
            .collect()
            .await;

        Ok(transactions)
    }

    async fn get_balance_changes_for_transaction<T>(&self, transaction_id: TransactionId) -> T
    where
        T: FromIterator<BalanceChange> + Default + Send + 'static,
    {
        self.query_map(
            "SELECT * FROM balance_changes WHERE transaction_id = ?",
            [transaction_id],
            Self::get_balance_change_from_row,
        )
        .await
        .unwrap_or(T::default())
    }

    fn get_account_from_row(row: &Row<'_>) -> Result<Account, rusqlite::Error> {
        let account = Account {
            address: row.get(0)?,
            id: row.get(1)?,
            name: row.get(2)?,
            network: row.get(3)?,
            derivation_path: row.get(4)?,
            public_key: Ed25519PublicKey(row.get(5)?),
            hidden: row.get(6)?,
            settings: row.get(7)?,
            balances_last_updated: row.get(8)?,
            transactions_last_updated: row.get(9)?,
        };
        Ok(account)
    }
    
    fn get_non_fungible_asset_from_row(row: &Row<'_>) -> Result<NonFungibleAsset, rusqlite::Error> {
        Ok(NonFungibleAsset {
            id: row.get(0)?,
            resource_address: row.get(1)?,
            nfids: row.get(2)?,
        })
    }
    
    fn get_fungible_asset_from_row(row: &Row<'_>) -> Result<FungibleAsset, rusqlite::Error> {
        Ok(FungibleAsset {
            id: row.get(0)?,
            resource_address: row.get(1)?,
            amount: row.get(2)?,
        })
    }

    fn get_resource_from_row(row: &Row<'_>) -> Result<Resource, rusqlite::Error> {
        let resource = Resource {
            address: row.get(0)?,
            name: row.get(1)?,
            symbol: row.get(2)?,
            description: row.get(3)?,
            current_supply: row.get(4)?,
            divisibility: row.get(5)?,
            tags: row.get(6)?,
        };
        Ok(resource)
    }

    fn get_transaction_from_row(row: &Row<'_>, balance_changes: Vec<BalanceChange>) -> Result<Transaction, rusqlite::Error> {
        Ok(Transaction {
            id: row.get(0)?,
            transaction_address: row.get(1)?,
            timestamp: row.get(2)?,
            state_version: row.get(3)?,
            balance_changes,
            message: row.get(4)?,
        })
    }

    fn get_transactionid_and_balance_change_from_row(
        row: &Row<'_>,
    ) -> Result<(TransactionId, BalanceChange), async_sqlite::rusqlite::Error> {
        let transaction_id: TransactionId = row.get(5)?;
        let balance_change = BalanceChange {
            id: row.get(0)?,
            account: row.get(1)?,
            resource: row.get(2)?,
            nfids: row.get(3)?,
            amount: row.get(4)?,
        };
        Ok((transaction_id, balance_change))
    }

    fn fold_transactions_and_balance_changes(
        mut map: HashMap<TransactionId, Vec<BalanceChange>>,
        (transaction_id, balance_change): (TransactionId, BalanceChange),
    ) -> HashMap<TransactionId, Vec<BalanceChange>> {
        map.entry(transaction_id)
            .or_insert(Vec::new())
            .push(balance_change);
        map
    }

    fn get_balance_change_from_row(row: &Row<'_>) -> Result<BalanceChange, rusqlite::Error> {
        Ok(BalanceChange {
            id: row.get(0)?,
            account: row.get(1)?,
            resource: row.get(2)?,
            nfids: row.get(3)?,
            amount: row.get(4)?,
        })
    }
}
