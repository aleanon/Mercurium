use once_cell::sync::OnceCell;
use types::{crypto::DataBaseKey, Network};

use crate::DbError;

pub static MAINNET_ICONCACHE: OnceCell<IconCache> = once_cell::sync::OnceCell::new();
pub static STOKENET_ICONCACHE: OnceCell<IconCache> = once_cell::sync::OnceCell::new();

pub struct IconCache {
    pub(crate) client: async_sqlite::Client,
}

impl IconCache {
    pub async fn load(network: Network, key: DataBaseKey) -> Result<&'static Self, DbError> {
        match network {
            Network::Mainnet => {
                let client = super::client::iconcache_client(network, key).await?;
                let icon_cache = Self { client };
                icon_cache.create_tables_if_not_exist().await?;
                let icon_cache = MAINNET_ICONCACHE.get_or_init(|| icon_cache);
                Ok(icon_cache)
            }
            Network::Stokenet => {
                let client = super::client::iconcache_client(network, key).await?;
                let icon_cache = Self { client };
                icon_cache.create_tables_if_not_exist().await?;

                let icon_cache = STOKENET_ICONCACHE.get_or_init(|| icon_cache);
                Ok(icon_cache)
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
