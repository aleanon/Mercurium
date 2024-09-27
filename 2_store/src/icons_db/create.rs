use super::statements::CREATE_ALL_ICONCACHE_TABLES_BATCH;
use crate::database::DbError;
use crate::IconsDb;

impl IconsDb {
    pub async fn create_tables_if_not_exist(&self) -> Result<(), DbError> {
        self.execute_batch(CREATE_ALL_ICONCACHE_TABLES_BATCH).await
    }
}
