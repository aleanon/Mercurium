pub mod create;
pub mod read;
pub mod statements;
pub mod update;

use crate::database::{DataBase, DbError};
use once_cell::sync::OnceCell;
use std::ops::Deref;
use types::{crypto::Key, AppPath, Network};

pub static MAINNET_DB: OnceCell<AppDataDb> = once_cell::sync::OnceCell::new();
pub static STOKENET_DB: OnceCell<AppDataDb> = once_cell::sync::OnceCell::new();

#[derive(Clone)]
pub struct AppDataDb {
    db: DataBase,
}
impl AppDataDb {
    pub async fn load(network: Network, key: Key<DataBase>) -> Result<&'static Self, DbError> {
        let app_data_db = Self::initialize(network, key).await?;
        app_data_db.create_tables_if_not_exist().await?;

        Ok(Self::get_static(network).get_or_init(|| app_data_db))
    }

    pub async fn initialize(network: Network, key: Key<DataBase>) -> Result<Self, DbError> {
        let app_path = AppPath::get();
        let path = app_path.db_path_ref(network);
        let db = DataBase::load(path, key).await?;
        Ok(Self { db })
    }

    pub async fn get_or_init(network: Network, key: Key<DataBase>) -> Result<&'static Self, DbError> {
        match Self::get(network) {
            Some(db) => Ok(db),
            None => Self::load(network, key).await,
        }
    }

    pub fn get(network: Network) -> Option<&'static Self> {
        Self::get_static(network).get()
    }

    pub fn exists(network: Network) -> bool {
        AppPath::get().db_path_ref(network).exists()
    }

    fn get_static(network: Network) -> &'static OnceCell<AppDataDb> {
        match network {
            Network::Mainnet => &MAINNET_DB,
            Network::Stokenet => &STOKENET_DB,
        }
    }
}

impl Deref for AppDataDb {
    type Target = DataBase;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
