use deps::async_sqlite::rusqlite::{self, Row, params};
use types::{Ed25519PublicKey, wallet_repository::AccountRepository};

use crate::sync_app_data_db::SyncAppDataDb;

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

impl AccountRepository for SyncAppDataDb {
    type Account = types::Account;
    type AccountId = types::address::AccountAddress;

    fn upsert_account(&mut self, account: Self::Account) -> Result<(), Self::Error> {
        let mut stmt = self.client.prepare_cached(UPSERT_ACCOUNT)?;
        stmt.execute(params![
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
    }

    fn upsert_accounts<Accounts: IntoIterator<Item = Self::Account>>(
        &mut self,
        accounts: Accounts,
    ) -> Result<(), Self::Error> {
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
    }

    fn get_account(&self, account_id: Self::AccountId) -> Result<Self::Account, Self::Error> {
        self.query_row(
            "SELECT * FROM accounts WHERE address = ?",
            [account_id],
            get_account_from_row,
        )
    }

    fn get_all_accounts<Accounts: FromIterator<Self::Account>>(
        &self,
    ) -> Result<Accounts, Self::Error> {
        self.query_map("SELECT * FROM accounts", [], get_account_from_row)
    }

    fn delete_account(&self, account_id: Self::AccountId) -> Result<(), Self::Error> {
        self.client
            .prepare_cached("DELETE FROM accounts WHERE address = ?")?
            .execute([account_id])?;
        Ok(())
    }
}

fn get_account_from_row(row: &Row<'_>) -> Result<types::Account, rusqlite::Error> {
    let account = types::Account {
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
