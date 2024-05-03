use anyhow::Result;
use thiserror::Error;

use types::app_path::{AppPath, AppPathError};

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Unable to create table {0}, source: {1}")]
    FailedToCreateTable(String, rusqlite::Error),
    #[error("{0}")]
    RusqliteError(#[from] rusqlite::Error),
    #[error("{0}")]
    TokioRusqliteError(#[from] tokio_rusqlite::Error),
    #[error("Failed to create Db connection, source: {0}")]
    FailedToCreateConnection(#[from] std::io::Error),
    #[error("Database not initialized")]
    DatabaseNotInitialized,
    #[error("No database found")]
    DatabaseNotFound,
    #[error("Database path not found")]
    UnableToEstablishPath(std::io::Error),
    #[error("Unable to establish path {0}")]
    PathError(#[from] AppPathError),
}

#[derive(Debug)]
pub struct Db {
    pub connection: rusqlite::Connection,
}

#[derive(Debug)]
pub struct AsyncDb {
    pub(crate) connection: tokio_rusqlite::Connection,
}

impl Db {
    pub fn new() -> Result<Db, DbError> {
        let connection = super::connection::connection_new_database()?;

        let mut db = Self { connection };

        db.create_all_tables()?;

        Ok(db)
    }

    pub fn placeholder() -> Db {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        Self { connection }
    }

    pub fn load() -> Result<Self, DbError> {
        let connection = super::connection::connection_existing_database()?;
        Ok(Self { connection })
    }

    pub fn initialize(&self) -> Result<()> {
        self.connection
            .execute("INSERT INTO initialized (is_initialized) VALUES TRUE", [])?;
        Ok(())
    }

    pub fn un_initialize(&self) -> Result<()> {
        self.connection
            .execute("INSERT INTO initialized DEFAULT VALUES", [])?;
        Ok(())
    }

    pub fn is_initialized(&self) -> Result<bool> {
        Ok(self.connection.query_row(
            "SELECT is_initialized FROM initialized WHERE id=1",
            [],
            |row| row.get::<usize, bool>(0),
        )?)
    }

    pub fn exits() -> Result<bool, AppPathError> {
        let apppath = AppPath::new()?;
        Ok(apppath.db_path().exists())
    }
}

impl AsyncDb {
    pub async fn load() -> Result<Self, DbError> {
        let connection = super::connection::async_connection_existing_database().await?;
        Ok(Self { connection })
    }

    pub fn exits() -> Result<bool, AppPathError> {
        let apppath = AppPath::new()?;
        Ok(apppath.db_path().exists())
    }

    pub async fn with_connection(connection: tokio_rusqlite::Connection) -> Self {
        Self { connection }
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    impl Db {
        pub fn with_connection(connection: rusqlite::Connection) -> Self {
            Self { connection }
        }
    }

    use types::Ed25519PublicKey;

    use super::*;
    use std::str::FromStr;
    use types::{
        Account, AccountAddress, Decimal, Fungible, Fungibles, MetaData, NFIDs, Network,
        NonFungible, NonFungibles, RadixDecimal, ResourceAddress,
    };

    #[test]
    fn test_accounts_table() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();

        let mut db = Db { connection };

        db.create_table_accounts().unwrap();
        db.create_table_fungibles().unwrap();

        let account = Account::new(
            1,
            "test_account 1".to_owned(),
            Network::Mainnet,
            [0u32; 6],
            AccountAddress::from_str(
                "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
            )
            .unwrap(),
            Ed25519PublicKey([0u8; Ed25519PublicKey::LENGTH]),
        );

        let mut fungibles = Fungibles::new();
        fungibles.insert(Fungible {
            name: "test fungible".to_owned(),
            symbol: "TF".to_owned(),
            icon: None,
            amount: Decimal::from(RadixDecimal::ONE_HUNDRED),
            total_supply: "1000".to_owned(),
            description: None,
            address: ResourceAddress::from_str(
                "resource_rdx1thlnv2lydu7np9w8guguqslkydv000d7ydn7uq0sestql96hrfml0v",
            )
            .unwrap(),
            last_updated_at_state_version: 150,
            metadata: MetaData::new(),
        });

        db.upsert_account(&account)
            .unwrap_or_else(|err| panic!("Error creating account, error: {err}"));
        db.update_fungibles_for_account(&fungibles, &account.address)
            .unwrap_or_else(|err| panic!("Error creating fungibles, error: {err}"));

        let accounts = db
            .get_accounts_map()
            .unwrap_or_else(|err| panic!("Unable to get accounts, error: {}", err));

        assert_eq!(accounts.get(&account.address), Some(&account))
    }

    #[test]
    fn test_fungibles_table() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();

        let mut db = Db { connection };

        db.create_table_accounts().unwrap();
        db.create_table_fungibles().unwrap();

        let account = Account::new(
            1,
            "test_account 1".to_owned(),
            Network::Mainnet,
            [0u32; 6],
            AccountAddress::from_str(
                "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
            )
            .unwrap(),
            Ed25519PublicKey([0u8; Ed25519PublicKey::LENGTH]),
        );

        let mut fungibles = Fungibles::new();
        fungibles.insert(Fungible {
            name: "test fungible".to_owned(),
            symbol: "TF".to_owned(),
            icon: None,
            amount: Decimal::from(RadixDecimal::ONE_HUNDRED),
            total_supply: "1000".to_owned(),
            description: None,
            address: ResourceAddress::from_str(
                "resource_rdx1thlnv2lydu7np9w8guguqslkydv000d7ydn7uq0sestql96hrfml0v",
            )
            .unwrap(),
            last_updated_at_state_version: 150,
            metadata: MetaData::new(),
        });

        db.upsert_account(&account)
            .unwrap_or_else(|err| panic!("Error creating account, error: {err}"));
        db.update_fungibles_for_account(&fungibles, &account.address)
            .unwrap_or_else(|err| panic!("Error creating fungibles, error: {err}"));

        let accounts = db
            .get_accounts_map()
            .unwrap_or_else(|err| panic!("Unable to get accounts, error: {}", err));

        assert_eq!(accounts.get(&account.address), Some(&account))
    }

    #[test]
    fn test_non_fungibles_table() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();

        let mut db = Db { connection };

        db.create_table_accounts().unwrap();
        db.create_table_non_fungibles().unwrap();

        let account = Account::new(
            1,
            "test_account 1".to_owned(),
            Network::Mainnet,
            [0u32; 6],
            AccountAddress::from_str(
                "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
            )
            .unwrap(),
            Ed25519PublicKey([0u8; Ed25519PublicKey::LENGTH]),
        );

        let mut non_fungibles = NonFungibles::new();
        non_fungibles.insert(NonFungible {
            name: "test nft".to_owned(),
            symbol: "TN".to_owned(),
            icon: None,
            description: None,
            nfids: NFIDs::new(),
            address: ResourceAddress::from_str(
                "resource_rdx1thlnv2lydu7np9w8guguqslkydv000d7ydn7uq0sestql96hrfml0v",
            )
            .unwrap(),
            last_updated_at_state_version: 140,
            metadata: MetaData::new(),
        });

        db.upsert_account(&account)
            .unwrap_or_else(|err| panic!("Error creating account, error: {err}"));
        db.update_non_fungibles_for_account(&non_fungibles, &account.address)
            .unwrap_or_else(|err| panic!("Error updating table: {}", err));

        let accounts = db
            .get_accounts_map()
            .unwrap_or_else(|err| panic!("Unable to get accounts, error: {}", err));

        assert_eq!(accounts.get(&account.address), Some(&account))
    }
}
