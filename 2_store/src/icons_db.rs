use deps::{async_sqlite::rusqlite::{params, Connection}, *};
use statements::resource_images::{CREATE_TABLE_RESOURCE_IMAGES, UPSERT_RESOURCE_IMAGE};

pub mod create;
pub mod read;
pub mod statements;
pub mod update;

use std::{collections::HashMap, ops::Deref};

use once_cell::sync::OnceCell;
use types::{address::ResourceAddress, crypto::Key, AppPath, Network};

use crate::{database::{DataBase, DbError}, SqliteKey};

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



pub struct SyncIconsDb;

impl SyncIconsDb {
    pub fn save_icons(icons: HashMap<ResourceAddress, Vec<u8>>, network: Network, key: &Key<DataBase>) -> Result<(), DbError> {
        let mut connection = Self::open_database_connection(network, key)?;
        connection.execute(CREATE_TABLE_RESOURCE_IMAGES, [])?;
        
        let tx = connection.transaction()?;
        {
            for (address, icons) in icons {
                tx.execute(UPSERT_RESOURCE_IMAGE, params![address, icons])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    fn open_database_connection(network: Network, key: &Key<DataBase>) -> Result<Connection, DbError>{
        let path = AppPath::get().db_path(network);
        let connection = async_sqlite::rusqlite::Connection::open(path)?;
        connection.pragma_update(None, "key", SqliteKey::from_key(key))?;
        Ok(connection)
    }
}
