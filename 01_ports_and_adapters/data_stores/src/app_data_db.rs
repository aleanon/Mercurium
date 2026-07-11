use deps::debug_print::debug_println;

pub mod create;
pub mod read;
pub mod statements;
pub mod update;

use crate::database::{DataBase, DbError};
use std::ops::Deref;
use types::{AppPathInner, Network, crypto::Key};

#[derive(Clone)]
pub struct AppDataDb {
    db: DataBase,
}

impl AppDataDb {
    /// Opens (and ensures tables exist in) the network's database with `key`.
    ///
    /// No process-global caching: the returned handle is **owned by the caller** — it lives in the
    /// `Unlocked` wallet state (produced at the `Locked → Unlocked` transition) and is cloned into
    /// any spawned task that needs it. See `.ai_docs/di_testability_plan.md` §1a.
    pub async fn open(
        paths: &AppPathInner,
        network: Network,
        key: Key<DataBase>,
    ) -> Result<Self, DbError> {
        let app_data_db = Self::initialize(paths, network, key).await?;
        app_data_db.create_tables_if_not_exist().await?;
        app_data_db.ensure_schema_version().await?;

        debug_println!("AppDataDb connection up");

        Ok(app_data_db)
    }

    pub async fn initialize(
        paths: &AppPathInner,
        network: Network,
        key: Key<DataBase>,
    ) -> Result<Self, DbError> {
        let path = paths.db_path_ref(network);
        let db = DataBase::load(path, key).await?;
        Ok(Self { db })
    }

    pub fn exists(paths: &AppPathInner, network: Network) -> bool {
        paths.db_path_ref(network).exists()
    }
}

impl Deref for AppDataDb {
    type Target = DataBase;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
