use std::ops::Deref;

use once_cell::sync::OnceCell;
use types::{crypto::DataBaseKey, AppPath, Network};

use crate::db::{DataBase, DbError};

pub static MAINNET_ICONCACHE: OnceCell<IconCache> = once_cell::sync::OnceCell::new();
pub static STOKENET_ICONCACHE: OnceCell<IconCache> = once_cell::sync::OnceCell::new();

pub struct IconCache {
    db: DataBase,
}

impl IconCache {
    pub async fn load(network: Network, key: DataBaseKey) -> Result<&'static Self, DbError> {
        let app_path = AppPath::get();
        match network {
            Network::Mainnet => {
                let path = app_path.icon_cache_ref(network);
                let db = DataBase::load(path, key).await?;
                let icons_db = Self { db };
                icons_db.create_tables_if_not_exist().await?;
                let icons_db = MAINNET_ICONCACHE.get_or_init(|| icons_db);
                Ok(icons_db)
            }
            Network::Stokenet => {
                let path = app_path.icon_cache_ref(network);
                let db = DataBase::load(path, key).await?;
                let icons_db = Self { db };
                icons_db.create_tables_if_not_exist().await?;

                let icons_db = STOKENET_ICONCACHE.get_or_init(|| icons_db);
                Ok(icons_db)
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
            Network::Mainnet => MAINNET_ICONCACHE.get(),
            Network::Stokenet => STOKENET_ICONCACHE.get(),
        }
    }
}

impl Deref for IconCache {
    type Target = DataBase;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
