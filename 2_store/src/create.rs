use super::statements::create::*;
use crate::db::DbError;
use crate::AppDataDb;
use crate::IconCache;

impl AppDataDb {
    pub async fn create_tables_if_not_exist(&self) -> Result<(), DbError> {
        Ok(self
            .conn(|conn| {
                conn.execute_batch(CREATE_ALL_MAIN_DB_TABLES_BATCH)?;
                Ok(())
            })
            .await?)
    }
}

impl IconCache {
    pub async fn create_tables_if_not_exist(&self) -> Result<(), DbError> {
        Ok(self
            .conn(|conn| {
                conn.execute_batch(CREATE_ALL_ICONCACHE_TABLES_BATCH)?;
                Ok(())
            })
            .await?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn execute_stmt(stmt: &str) -> Result<(), async_sqlite::Error> {
        let client = async_sqlite::ClientBuilder::new().open_blocking().unwrap();
        let stmt = stmt.to_owned();
        client
            .conn_blocking(move |conn| conn.execute(stmt.as_str(), []))
            .map(|_| ())
    }

    fn execute_batch_stmt(stmt: &str) -> Result<(), async_sqlite::Error> {
        let client = async_sqlite::ClientBuilder::new().open_blocking().unwrap();
        let stmt = stmt.to_owned();
        client
            .conn_blocking(move |conn| conn.execute_batch(stmt.as_str()))
            .map(|_| ())
    }

    #[test]
    fn test_create_all_tables_main_db() {
        let result = execute_batch_stmt(CREATE_ALL_MAIN_DB_TABLES_BATCH);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_accounts() {
        let result = execute_stmt(CREATE_TABLE_ACCOUNTS);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_password_hash() {
        let result = execute_stmt(CREATE_TABLE_PASSWORD_HASH);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_resources() {
        let result = execute_stmt(CREATE_TABLE_RESOURCES);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_fungible_assets() {
        let result = execute_stmt(CREATE_TABLE_FUNGIBLE_ASSETS);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_non_fungible_assets() {
        let result = execute_stmt(CREATE_TABLE_NON_FUNGIBLE_ASSETS);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_balance_changes() {
        let result = execute_stmt(CREATE_TABLE_BALANCE_CHANGES);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_transactions() {
        let result = execute_stmt(CREATE_TABLE_TRANSACTIONS);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_all_tables_iconcache() {
        let result = execute_batch_stmt(CREATE_ALL_ICONCACHE_TABLES_BATCH);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_resource_images() {
        let result = execute_stmt(CREATE_TABLE_RESOURCE_IMAGES);
        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_table_nft_images() {
        let result = execute_stmt(CREATE_TABLE_NFT_IMAGES);
        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
