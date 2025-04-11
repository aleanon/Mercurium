use deps_two::*;

pub mod create;
pub mod read;
pub mod statements;
pub mod update;

use std::ops::Deref;

use once_cell::sync::OnceCell;
use types::{crypto::Key, AppPath, Network};

use crate::database::{DataBase, DbError};

pub static MAINNET_ICONCACHE: OnceCell<IconsDb> = OnceCell::new();
pub static STOKENET_ICONCACHE: OnceCell<IconsDb> = OnceCell::new();

pub struct IconsDb {
    db: DataBase,
}

impl IconsDb {
    pub async fn load(network: Network, key: Key<DataBase>) -> Result<&'static Self, DbError> {
        let icons_db = Self::initialize(network, key).await?;
        icons_db.create_tables_if_not_exist().await?;

        Ok(Self::get_static(network).get_or_init(|| icons_db))
    }

    pub async fn initialize(network: Network, key: Key<DataBase>) -> Result<Self, DbError> {
        let app_path = AppPath::get();
        let path = app_path.icon_cache_ref(network);
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
        match network {
            Network::Mainnet => MAINNET_ICONCACHE.get(),
            Network::Stokenet => STOKENET_ICONCACHE.get(),
        }
    }

    fn get_static(network: Network) -> &'static OnceCell<IconsDb> {
        match network {
            Network::Mainnet => &MAINNET_ICONCACHE,
            Network::Stokenet => &STOKENET_ICONCACHE,
        }
    }
}

impl Deref for IconsDb {
    type Target = DataBase;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
