use once_cell::sync::OnceCell;
use thiserror::Error;

use types::{
    crypto::DataBaseKey,
    Network, {AppPath, AppPathError},
};

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Unable to create table {0}, source: {1}")]
    FailedToCreateTable(String, rusqlite::Error),
    #[error("{0}")]
    RusqliteError(#[from] rusqlite::Error),
    #[error("{0}")]
    AsyncSqliteError(#[from] async_sqlite::Error),
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
    #[error("Invalid password")]
    InvalidPassword,
}

pub static MAINNET_DB: OnceCell<AsyncDb> = once_cell::sync::OnceCell::new();
pub static STOKENET_DB: OnceCell<AsyncDb> = once_cell::sync::OnceCell::new();

#[derive(Debug)]
pub struct Db {
    pub connection: rusqlite::Connection,
}

#[derive(Clone)]
pub struct AsyncDb {
    pub(crate) client: async_sqlite::Client,
}

impl Db {
    pub fn new(network: Network, key: &DataBaseKey) -> Result<Db, DbError> {
        let connection = super::connection::connection_new_database(network, key)?;

        let db = Self { connection };

        db.create_tables_if_not_exist()?;

        Ok(db)
    }

    pub fn load(network: Network, key: &DataBaseKey) -> Result<Self, DbError> {
        let connection = super::connection::connection_existing_database(network, key)?;
        Ok(Self { connection })
    }

    pub fn exists(network: Network) -> bool {
        AppPath::get().db_path_ref(network).exists()
    }

    #[cfg(not(release))]
    pub fn new_in_memory() -> Db {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        Self { connection }
    }
}

impl AsyncDb {
    // pub async fn new(network: Network, key: DataBaseKey) -> Result<Self, DbError> {
    //     let connection = super::connection::async_connection(network, key).await?;
    //     let db = Self { client: connection };
    //     db.create_tables_if_not_exist().await?;

    //     match network {
    //         Network::Mainnet => {
    //             MAINNET_DB.set(db.clone()).ok();
    //         }
    //         Network::Stokenet => {
    //             STOKENET_DB.set(db.clone()).ok();
    //         }
    //     }
    //     Ok(db)
    // }

    pub async fn load(network: Network, key: DataBaseKey) -> Result<&'static Self, DbError> {
        match network {
            Network::Mainnet => {
                let connection = super::connection::async_connection(network, key).await?;
                let db = Self { client: connection };
                db.create_tables_if_not_exist().await?;
                let db = MAINNET_DB.get_or_init(|| db);
                Ok(db)
            }
            Network::Stokenet => {
                let client = super::connection::async_connection(network, key).await?;
                let db = Self { client };
                db.create_tables_if_not_exist().await?;

                let db = STOKENET_DB.get_or_init(|| db);
                Ok(db)
            }
        }
    }

    pub async fn get_or_init(network: Network, key: DataBaseKey) -> Result<&'static Self, DbError> {
        match Self::get(network) {
            Some(db) => Ok(db),
            None => Self::load(network, key).await,
        }
    }

    pub fn get(network: Network) -> Option<&'static Self> {
        match network {
            Network::Mainnet => MAINNET_DB.get(),
            Network::Stokenet => STOKENET_DB.get(),
        }
    }

    pub fn exists(network: Network) -> bool {
        AppPath::get().db_path_ref(network).exists()
    }

    pub fn with_connection(client: async_sqlite::Client) -> Self {
        Self { client }
    }
}

// #[cfg(test)]
// mod tests {
//     #![allow(dead_code)]

//     impl Db {
//         pub fn with_connection(connection: rusqlite::Connection) -> Self {
//             Self { connection }
//         }
//     }

//     use types::Ed25519PublicKey;

//     use super::*;
//     use std::{collections::HashMap, str::FromStr};
//     use types::{
//         Account, AccountAddress, Decimal, Fungible, Fungibles, MetaData, NFIDs, Network,
//         NonFungible, NonFungibles, RadixDecimal, ResourceAddress,
//     };

//     #[test]
//     fn test_accounts_table() {
//         let connection = rusqlite::Connection::open_in_memory().unwrap();

//         let mut db = Db { connection };

//         db.create_table_accounts().unwrap();
//         db.create_table_fungibles().unwrap();

//         let account = Account::new(
//             1,
//             "test_account 1".to_owned(),
//             Network::Mainnet,
//             [0u32; 6],
//             AccountAddress::from_str(
//                 "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
//             )
//             .unwrap(),
//             Ed25519PublicKey([0u8; Ed25519PublicKey::LENGTH]),
//         );

//         let mut fungibles = Fungibles::new();
//         fungibles.insert(Fungible {
//             name: "test fungible".to_owned(),
//             symbol: "TF".to_owned(),
//             icon: None,
//             amount: Decimal::from(RadixDecimal::ONE_HUNDRED),
//             total_supply: "1000".to_owned(),
//             description: None,
//             address: ResourceAddress::from_str(
//                 "resource_rdx1thlnv2lydu7np9w8guguqslkydv000d7ydn7uq0sestql96hrfml0v",
//             )
//             .unwrap(),
//             last_updated_at_state_version: 150,
//             metadata: MetaData::new(),
//         });

//         db.upsert_account(&account)
//             .unwrap_or_else(|err| panic!("Error creating account, error: {err}"));
//         db.update_fungibles_for_account(&fungibles, &account.address)
//             .unwrap_or_else(|err| panic!("Error creating fungibles, error: {err}"));

//         let accounts: HashMap<AccountAddress, Account> = db
//             .get_accounts()
//             .unwrap_or_else(|err| panic!("Unable to get accounts, error: {}", err));

//         assert_eq!(accounts.get(&account.address), Some(&account))
//     }

//     #[test]
//     fn test_fungibles_table() {
//         let connection = rusqlite::Connection::open_in_memory().unwrap();

//         let mut db = Db { connection };

//         db.create_table_accounts().unwrap();
//         db.create_table_fungibles().unwrap();

//         let account = Account::new(
//             1,
//             "test_account 1".to_owned(),
//             Network::Mainnet,
//             [0u32; 6],
//             AccountAddress::from_str(
//                 "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
//             )
//             .unwrap(),
//             Ed25519PublicKey([0u8; Ed25519PublicKey::LENGTH]),
//         );

//         let mut fungibles = Fungibles::new();
//         fungibles.insert(Fungible {
//             name: "test fungible".to_owned(),
//             symbol: "TF".to_owned(),
//             icon: None,
//             amount: Decimal::from(RadixDecimal::ONE_HUNDRED),
//             total_supply: "1000".to_owned(),
//             description: None,
//             address: ResourceAddress::from_str(
//                 "resource_rdx1thlnv2lydu7np9w8guguqslkydv000d7ydn7uq0sestql96hrfml0v",
//             )
//             .unwrap(),
//             last_updated_at_state_version: 150,
//             metadata: MetaData::new(),
//         });

//         db.upsert_account(&account)
//             .unwrap_or_else(|err| panic!("Error creating account, error: {err}"));
//         db.update_fungibles_for_account(&fungibles, &account.address)
//             .unwrap_or_else(|err| panic!("Error creating fungibles, error: {err}"));

//         let accounts: HashMap<AccountAddress, Account> = db
//             .get_accounts()
//             .unwrap_or_else(|err| panic!("Unable to get accounts, error: {}", err));

//         assert_eq!(accounts.get(&account.address), Some(&account))
//     }

//     #[test]
//     fn test_non_fungibles_table() {
//         let connection = rusqlite::Connection::open_in_memory().unwrap();

//         let mut db = Db { connection };

//         db.create_table_accounts().unwrap();
//         db.create_table_non_fungibles().unwrap();

//         let account = Account::new(
//             1,
//             "test_account 1".to_owned(),
//             Network::Mainnet,
//             [0u32; 6],
//             AccountAddress::from_str(
//                 "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
//             )
//             .unwrap(),
//             Ed25519PublicKey([0u8; Ed25519PublicKey::LENGTH]),
//         );

//         let mut non_fungibles = NonFungibles::new();
//         non_fungibles.insert(NonFungible {
//             name: "test nft".to_owned(),
//             symbol: "TN".to_owned(),
//             icon: None,
//             description: None,
//             nfids: NFIDs::new(),
//             address: ResourceAddress::from_str(
//                 "resource_rdx1thlnv2lydu7np9w8guguqslkydv000d7ydn7uq0sestql96hrfml0v",
//             )
//             .unwrap(),
//             last_updated_at_state_version: 140,
//             metadata: MetaData::new(),
//         });

//         db.upsert_account(&account)
//             .unwrap_or_else(|err| panic!("Error creating account, error: {err}"));
//         db.update_non_fungibles_for_account(&non_fungibles, &account.address)
//             .unwrap_or_else(|err| panic!("Error updating table: {}", err));

//         let accounts: HashMap<AccountAddress, Account> = db
//             .get_accounts()
//             .unwrap_or_else(|err| panic!("Unable to get accounts, error: {}", err));

//         assert_eq!(accounts.get(&account.address), Some(&account))
//     }
// }
