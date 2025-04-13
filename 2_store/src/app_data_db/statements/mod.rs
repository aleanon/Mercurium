use deps::const_format;

pub mod accounts;
pub mod balance_changes;
pub mod fungible_assets;
pub mod non_fungible_assets;
pub mod password_hash;
pub mod resources;
pub mod transaction;

use self::{
    accounts::CREATE_TABLE_ACCOUNTS, balance_changes::CREATE_TABLE_BALANCE_CHANGES,
    fungible_assets::CREATE_TABLE_FUNGIBLE_ASSETS,
    non_fungible_assets::CREATE_TABLE_NON_FUNGIBLE_ASSETS,
    password_hash::CREATE_TABLE_PASSWORD_HASH, resources::CREATE_TABLE_RESOURCES,
    transaction::CREATE_TABLE_TRANSACTIONS,
};

pub const CREATE_ALL_MAIN_DB_TABLES_BATCH: &'static str = const_format::formatcp!(
    "BEGIN;
    {CREATE_TABLE_PASSWORD_HASH};
    {CREATE_TABLE_ACCOUNTS};
    {CREATE_TABLE_RESOURCES};
    {CREATE_TABLE_FUNGIBLE_ASSETS};
    {CREATE_TABLE_NON_FUNGIBLE_ASSETS};
    {CREATE_TABLE_TRANSACTIONS};
    {CREATE_TABLE_BALANCE_CHANGES};
    COMMIT;"
);

#[cfg(test)]
mod test {
    use deps::*;

    use crate::database::test::{execute_batch_stmt, execute_stmt};

    use super::*;

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
}
