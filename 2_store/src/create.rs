use super::statements::create::*;
use crate::Db;
use crate::IconCache;

impl Db {
    pub async fn create_tables_if_not_exist(&self) -> Result<(), async_sqlite::Error> {
        self.client
            .conn(|conn| {
                conn.execute_batch(CREATE_ALL_MAIN_DB_TABLES_BATCH)?;
                Ok(())
            })
            .await
    }
}

impl IconCache {
    pub async fn create_tables_if_not_exist(&self) -> Result<(), async_sqlite::Error> {
        self.client
            .conn(|conn| {
                conn.execute_batch(CREATE_ALL_ICONCACHE_TABLES_BATCH)?;
                Ok(())
            })
            .await
    }
}
