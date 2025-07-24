mod accounts;
mod resources;

use std::ops::{Deref, DerefMut};

use types::{crypto::Key, repository::Repository};

use crate::{
    DataBase, DbError, app_data_db::statements::CREATE_ALL_MAIN_DB_TABLES_BATCH,
    database::SyncDataBase,
};

pub struct SyncAppDataDb {
    db: SyncDataBase,
}

impl Deref for SyncAppDataDb {
    type Target = SyncDataBase;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl DerefMut for SyncAppDataDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl Repository for SyncAppDataDb {
    type Key = Key<DataBase>;
    type Error = DbError;
    type Path = &'static Box<std::path::Path>;

    fn connect(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error> {
        let db = SyncDataBase::load(path, key)?;
        Ok(Self { db })
    }

    fn initialize(path: Self::Path, key: Self::Key) -> Result<Self, Self::Error> {
        let db = Self::connect(path, key)?;
        db.client.execute_batch(CREATE_ALL_MAIN_DB_TABLES_BATCH)?;
        Ok(db)
    }

    fn delete(path: Self::Path, key: Self::Key) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
