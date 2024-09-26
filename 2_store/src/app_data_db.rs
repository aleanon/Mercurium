use crate::db::{DataBase, DbError};
use once_cell::sync::OnceCell;
use std::ops::Deref;
use types::{crypto::DataBaseKey, AppPath, Network};

pub static MAINNET_DB: OnceCell<AppDataDb> = once_cell::sync::OnceCell::new();
pub static STOKENET_DB: OnceCell<AppDataDb> = once_cell::sync::OnceCell::new();

#[derive(Clone)]
pub struct AppDataDb {
    db: DataBase,
}
impl AppDataDb {
    pub async fn load(network: Network, key: DataBaseKey) -> Result<&'static Self, DbError> {
        let app_path = AppPath::get();
        match network {
            Network::Mainnet => {
                let path = app_path.db_path_ref(network);
                let db = DataBase::load(path, key).await?;
                let app_data_db = Self { db };
                app_data_db.create_tables_if_not_exist().await?;
                let app_data_db = MAINNET_DB.get_or_init(|| app_data_db);
                Ok(app_data_db)
            }
            Network::Stokenet => {
                let path = app_path.db_path_ref(network);
                let db = DataBase::load(path, key).await?;
                let app_data_db = Self { db };
                app_data_db.create_tables_if_not_exist().await?;

                let app_data_db = STOKENET_DB.get_or_init(|| app_data_db);
                Ok(app_data_db)
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
}

impl Deref for AppDataDb {
    type Target = DataBase;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
