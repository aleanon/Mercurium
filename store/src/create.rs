use super::{
    db::{Db, DbError},
    statements::create::*,
};
use anyhow::Result;

use super::db::AsyncDb;

impl Db {
    pub fn create_tables_if_not_exist(&mut self) -> Result<(), DbError> {
        self.create_table_accounts()?;
        self.create_table_resources()?;
        self.create_table_fungible_assets()?;
        self.create_table_non_fungible_assets()?;
        self.create_table_transactions()?;
        self.create_table_balance_changes()?;
        Ok(())
    }

    pub fn create_table_accounts(&self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_ACCOUNTS, [])
            .map_err(|err| DbError::FailedToCreateTable("accounts".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_resources(&self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_RESOURCES, [])
            .map_err(|err| DbError::FailedToCreateTable("resources".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_fungible_assets(&self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_FUNGIBLE_ASSETS, [])
            .map_err(|err| DbError::FailedToCreateTable("fungible assets".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_non_fungible_assets(&self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_NON_FUNGIBLE_ASSETS, [])
            .map_err(|err| DbError::FailedToCreateTable("non_fungible_assets".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_transactions(&self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_TRANSACTIONS, [])
            .map_err(|err| DbError::FailedToCreateTable("transactions".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_balance_changes(&self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_BALANCE_CHANGES, [])
            .map_err(|err| DbError::FailedToCreateTable("balance_changes".to_owned(), err))?;
        Ok(())
    }
}

impl AsyncDb {
    pub async fn create_table_accounts(&mut self) -> Result<(), DbError> {
        self.connection
            .call_unwrap(|conn| conn.execute(CREATE_TABLE_ACCOUNTS, []))
            .await
            .map_err(|err| DbError::FailedToCreateTable("accounts".to_owned(), err))?;
        Ok(())
    }

    pub async fn create_table_resources(&mut self) -> Result<(), DbError> {
        self.connection
            .call_unwrap(|conn| conn.execute(CREATE_TABLE_RESOURCES, []))
            .await
            .map_err(|err| DbError::FailedToCreateTable("resources".to_owned(), err))?;
        Ok(())
    }

    pub async fn create_table_transactions(&mut self) -> Result<(), DbError> {
        self.connection
            .call_unwrap(|conn| conn.execute(CREATE_TABLE_TRANSACTIONS, []))
            .await
            .map_err(|err| DbError::FailedToCreateTable("transactions".to_owned(), err))?;
        Ok(())
    }

    pub async fn create_table_fungible_assets(&mut self) -> Result<(), DbError> {
        self.connection
            .call_unwrap(|conn| conn.execute(CREATE_TABLE_FUNGIBLE_ASSETS, []))
            .await
            .map_err(|err| DbError::FailedToCreateTable("fungible assets".to_owned(), err))?;
        Ok(())
    }

    pub async fn create_table_non_fungible_assets(&mut self) -> Result<(), DbError> {
        self.connection
            .call_unwrap(|conn| conn.execute(CREATE_TABLE_NON_FUNGIBLE_ASSETS, []))
            .await
            .map_err(|err| DbError::FailedToCreateTable("non_fungible_assets".to_owned(), err))?;
        Ok(())
    }
}
