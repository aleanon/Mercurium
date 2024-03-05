use super::{statements::create::*,  db::{Db, DbError}};
use anyhow::Result;

use super::db::AsyncDb;


impl Db {
    pub fn create_table_accounts(&mut self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_ACCOUNTS, [])
            .map_err(|err| DbError::FailedToCreateTable("accounts".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_resources(&mut self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_RESOURCES, [])
            .map_err(|err| DbError::FailedToCreateTable("resources".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_transactions(&mut self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_TRANSACTIONS, [])
            .map_err(|err| DbError::FailedToCreateTable("transactions".to_owned(), err))?;
        Ok(())
    }

    pub async fn create_table_fungible_assets(&mut self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_FUNGIBLE_ASSETS, [])
            .map_err(|err| DbError::FailedToCreateTable("fungible assets".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_non_fungible_assets(&mut self) -> Result<(), DbError> {
        self.connection
            .execute(CREATE_TABLE_NON_FUNGIBLE_ASSETS, [])
            .map_err(|err| DbError::FailedToCreateTable("non_fungible_assets".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_fungibles(&mut self) -> Result<(), DbError> {
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS fungibles (
                address BLOB NOT NULL PRIMARY KEY,
                name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                icon BLOB,
                amount TEXT NOT NULL,
                current_supply TEXT NOT NULL,
                description TEXT,
                last_updated INTEGER NOT NULL,
                metadata BLOB NOT NULL,
                account_address BLOB NOT NULL,
                FOREIGN KEY(account_address) REFERENCES accounts(address)
            )",
                [],
            )
            .map_err(|err| DbError::FailedToCreateTable("fungibles".to_owned(), err))?;
        Ok(())
    }

    pub fn create_table_non_fungibles(&mut self) -> Result<(), DbError> {
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS non_fungibles (
                address BLOB NOT NULL PRIMARY KEY,
                name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                icon BLOB,
                description TEXT,
                nfids BLOB NOT NULL,
                last_updated INTEGER NOT NULL,
                metadata BLOB NOT NULL,
                account_address BLOB NOT NULL, 
                FOREIGN KEY(account_address) REFERENCES accounts(address)
            )",
                [],
            )
            .map_err(|err| DbError::FailedToCreateTable("non_fungible".to_owned(), err))?;
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

    pub async fn create_table_fungibles(&mut self) -> Result<(), DbError> {
        self.connection
            .call_unwrap(|conn| {
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS fungibles (
                address BLOB NOT NULL PRIMARY KEY,
                name TEXT,
                symbol TEXT,
                icon BLOB,
                amount TEXT NOT NULL,
                current_supply TEXT NOT NULL,
                description TEXT,
                last_updated INTEGER NOT NULL,
                metadata BLOB NOT NULL,
                account_address TEXT NOT NULL,
                FOREIGN KEY(account_address) REFERENCES accounts(address)
            )",
                    [],
                )
            })
            .await
            .map_err(|err| DbError::FailedToCreateTable("fungibles".to_owned(), err))?;
        Ok(())
    }

    pub async fn create_table_non_fungibles(&mut self) -> rusqlite::Result<(), DbError> {
        self.connection
            .call_unwrap(|conn| {
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS non_fungibles (
                address BLOB NOT NULL PRIMARY KEY,
                name TEXT,
                symbol TEXT,
                icon BLOB,
                description TEXT,
                nfids BLOB NOT NULL,
                last_updated INTEGER NOT NULL,
                metadata BLOB NOT NULL,
                account_address TEXT NOT NULL, 
                FOREIGN KEY(account_address) REFERENCES accounts(address)
            )",
                    [],
                )
            })
            .await
            .map_err(|err| DbError::FailedToCreateTable("non_fungible".to_owned(), err))?;
        Ok(())
    }
}
