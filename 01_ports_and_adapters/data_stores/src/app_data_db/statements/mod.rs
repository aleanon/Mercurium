use deps::const_format;

pub mod accounts;
pub mod balance_changes;
pub mod fungible_assets;
pub mod non_fungible_assets;
pub mod password_hash;
pub mod personas;
pub mod resources;
pub mod transaction;

use self::{
    accounts::CREATE_TABLE_ACCOUNTS, balance_changes::CREATE_TABLE_BALANCE_CHANGES,
    fungible_assets::CREATE_TABLE_FUNGIBLE_ASSETS,
    non_fungible_assets::CREATE_TABLE_NON_FUNGIBLE_ASSETS,
    password_hash::CREATE_TABLE_PASSWORD_HASH, personas::CREATE_TABLE_PERSONAS,
    resources::CREATE_TABLE_RESOURCES, transaction::CREATE_TABLE_TRANSACTIONS,
};

pub const CREATE_ALL_MAIN_DB_TABLES_BATCH: &'static str = const_format::formatcp!(
    "BEGIN;
    {CREATE_TABLE_PASSWORD_HASH};
    {CREATE_TABLE_ACCOUNTS};
    {CREATE_TABLE_PERSONAS};
    {CREATE_TABLE_RESOURCES};
    {CREATE_TABLE_FUNGIBLE_ASSETS};
    {CREATE_TABLE_NON_FUNGIBLE_ASSETS};
    {CREATE_TABLE_TRANSACTIONS};
    {CREATE_TABLE_BALANCE_CHANGES};
    COMMIT;"
);

#[cfg(test)]
mod test {

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
    fn test_create_table_personas() {
        let result = execute_stmt(CREATE_TABLE_PERSONAS);
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
    fn transaction_and_balance_change_write_path_is_valid_sql() {
        use deps::async_sqlite::{ClientBuilder, rusqlite::params};

        let client = ClientBuilder::new().open_blocking().unwrap();
        client
            .conn_blocking(|conn| {
                conn.execute(super::transaction::CREATE_TABLE_TRANSACTIONS, [])?;
                conn.execute(super::balance_changes::CREATE_TABLE_BALANCE_CHANGES, [])?;

                // Insert a transaction, then a balance change referencing it (FK).
                conn.prepare_cached(super::transaction::UPSERT_TRANSACTION)?
                    .execute(params![vec![1u8], vec![2u8], vec![3u8], 10i64, Some("msg")])?;
                conn.prepare_cached(super::balance_changes::INSERT_BALANCE_CHANGE)?
                    .execute(params![
                        vec![9u8],
                        vec![1u8],
                        vec![2u8],
                        Option::<Vec<u8>>::None,
                        Some("5"),
                        vec![1u8]
                    ])?;

                // Upsert the same transaction (ON CONFLICT update) and replace its balance change.
                conn.prepare_cached(super::transaction::UPSERT_TRANSACTION)?
                    .execute(params![vec![1u8], vec![2u8], vec![3u8], 11i64, Some("msg2")])?;
                conn.prepare_cached(super::balance_changes::INSERT_BALANCE_CHANGE)?
                    .execute(params![
                        vec![9u8],
                        vec![1u8],
                        vec![2u8],
                        Option::<Vec<u8>>::None,
                        Some("6"),
                        vec![1u8]
                    ])?;
                Ok(())
            })
            .expect("the transaction/balance-change statements match the schema");
    }

    #[test]
    fn transaction_and_balance_change_read_queries_are_valid_sql() {
        use deps::async_sqlite::ClientBuilder;

        // Preparing a query validates its table/column references against the schema, so this
        // catches column-name drift (e.g. `account_address` vs `account`, `transaction_id` vs
        // `tx_id`/`id`) that previously surfaced only at runtime as "no such column".
        let client = ClientBuilder::new().open_blocking().unwrap();
        client
            .conn_blocking(|conn| {
                conn.execute(super::transaction::CREATE_TABLE_TRANSACTIONS, [])?;
                conn.execute(super::balance_changes::CREATE_TABLE_BALANCE_CHANGES, [])?;

                conn.prepare(super::transaction::SELECT_TRANSACTION_BY_ID)?;
                conn.prepare(super::balance_changes::SELECT_BALANCE_CHANGES_BY_ACCOUNT)?;
                conn.prepare(super::balance_changes::SELECT_BALANCE_CHANGES_BY_TX_ID)?;
                Ok(())
            })
            .expect("the transaction/balance-change read queries match the schema");
    }
}
