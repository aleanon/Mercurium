use async_trait::async_trait;
use deps::async_sqlite::rusqlite::{self, Row, params};
use types::Ed25519PublicKey;

use crate::{
    adapters::data_store::sqlite::Sqlite,
    ports::wallet_data_store::{AccountStore, WalletDataStoreError},
};

type Account = types::Account;
type AccountId = types::address::AccountAddress;

pub const CREATE_TABLE_ACCOUNTS: &'static str = "CREATE TABLE IF NOT EXISTS
    accounts (
        address BLOB NOT NULL PRIMARY KEY,
        id INTEGER NOT NULL,
        name TEXT NOT NULL,
        network INTEGER NOT NULL,
        derivation_path BLOB NOT NULL,
        public_key BLOB NOT NULL,
        hidden BOOL NOT NULL,
        settings BLOB NOT NULL,
        balances_last_updated INTEGER,
        transactions_last_updated INTEGER
    )
";

pub const UPSERT_ACCOUNT: &'static str = "INSERT INTO
    accounts (
        address,
        id,
        name,
        network,
        derivation_path,
        public_key,
        hidden,
        settings,
        balances_last_updated,
        transactions_last_updated
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT (address)
    DO UPDATE SET
        id = excluded.id,
        name = excluded.name,
        network = excluded.network,
        derivation_path = excluded.derivation_path,
        public_key = excluded.public_key,
        hidden = excluded.hidden,
        settings = excluded.settings,
        balances_last_updated = excluded.balances_last_updated,
        transactions_last_updated = excluded.transactions_last_updated
";

#[async_trait]
impl AccountStore for Sqlite {
    async fn upsert_account(&mut self, account: Account) -> Result<(), WalletDataStoreError> {
        self.transaction(UPSERT_ACCOUNT, move |cached_stmt| {
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

    async fn upsert_accounts<Accounts>(
        &mut self,
        accounts: Accounts,
    ) -> Result<(), WalletDataStoreError>
    where
        Accounts: IntoIterator<Item = Account> + Send + 'static,
    {
        self.transaction(UPSERT_ACCOUNT, move |cached_stmt| {
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
        .await?;
        Ok(())
    }

    async fn get_account(&self, account_id: AccountId) -> Result<Account, WalletDataStoreError> {
        Ok(self
            .query_row(
                "SELECT * FROM accounts WHERE address = ?",
                [account_id],
                get_account_from_row,
            )
            .await?)
    }

    async fn get_all_accounts<Accounts>(&self) -> Result<Accounts, WalletDataStoreError>
    where
        Accounts: FromIterator<Account> + Send + 'static,
    {
        self.query_map("SELECT * FROM accounts", [], get_account_from_row)
            .await
            .map_err(Into::into)
    }

    async fn get_account_addresses<T>(&self) -> Result<T, WalletDataStoreError>
    where
        T: FromIterator<AccountId> + Send + 'static,
    {
        self.query_map("SELECT address FROM accounts", [], |row| Ok(row.get(0)?))
            .await
            .map_err(Into::into)
    }

    async fn delete_account(&self, account_id: AccountId) -> Result<(), WalletDataStoreError> {
        self.prepare_cached_statement("DELETE FROM accounts WHERE address = ?", |stmt| {
            stmt.execute([account_id])
        })
        .await?;
        Ok(())
    }
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
