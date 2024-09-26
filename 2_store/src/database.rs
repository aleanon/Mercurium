use std::path::Path;

use async_sqlite::rusqlite::{self, CachedStatement, Connection, OpenFlags, Params, Result, Row};
use debug_print::debug_println;
use thiserror::Error;
use types::{crypto::DataBaseKey, AppPathError};

#[derive(Debug, Error)]
pub enum DbError {
    #[error("{0}")]
    AsyncSqliteError(#[from] async_sqlite::Error),
    // #[error("Failed to create Db connection, source: {0}")]
    // FailedToCreateConnection(#[from] std::io::Error),
    #[error("Database not initialized")]
    DatabaseNotInitialized,
    #[error("No database found")]
    DatabaseNotFound,
    #[error("Unable to establish path {0}")]
    PathError(#[from] AppPathError),
}

impl From<rusqlite::Error> for DbError {
    fn from(value: rusqlite::Error) -> Self {
        Self::AsyncSqliteError(async_sqlite::Error::Rusqlite(value))
    }
}

#[derive(Clone)]
pub struct DataBase {
    pub(crate) client: async_sqlite::Client,
}

impl DataBase {
    pub(crate) async fn load(path: &Path, key: DataBaseKey) -> Result<Self, DbError> {
        let client = Self::build_async_db_client(path).await?;
        Self::set_database_key(&client, key).await?;

        debug_println!("AsyncDb connection up");

        Ok(Self { client })
    }

    async fn build_async_db_client(
        path: &Path,
    ) -> Result<async_sqlite::Client, async_sqlite::Error> {
        async_sqlite::ClientBuilder::new()
            .path(path)
            .flags(OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE)
            .open()
            .await
    }

    async fn set_database_key(
        client: &async_sqlite::Client,
        key: DataBaseKey,
    ) -> Result<(), async_sqlite::Error> {
        client
            .conn(move |conn| conn.pragma_update(None, "key", key))
            .await
    }

    pub(crate) async fn conn<T, F>(&self, f: F) -> Result<T, async_sqlite::Error>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client.conn(f).await
    }

    pub(crate) async fn conn_mut<T, F>(&self, f: F) -> Result<T, async_sqlite::Error>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client.conn_mut(f).await
    }

    pub(crate) async fn transaction<F>(
        &self,
        stmt: &'static str,
        execute_stmt: F,
    ) -> Result<(), async_sqlite::Error>
    where
        F: FnOnce(&mut CachedStatement) -> Result<(), rusqlite::Error> + Send + 'static,
    {
        self.conn_mut(|conn| {
            let tx = conn.transaction()?;

            execute_stmt(&mut tx.prepare_cached(stmt)?)?;

            tx.commit()
        })
        .await
    }

    pub(crate) async fn prepare_cached_statement<T, F>(
        &self,
        stmt: &'static str,
        func: F,
    ) -> Result<T, async_sqlite::Error>
    where
        F: FnOnce(&mut CachedStatement<'_>) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        Ok(self
            .conn_mut(|conn| {
                let mut cached_statement = conn.prepare_cached(stmt)?;
                func(&mut cached_statement)
            })
            .await?)
    }

    pub(crate) async fn query_row<T, P, F>(
        &self,
        stmt: &'static str,
        params: P,
        f: F,
    ) -> Result<T, DbError>
    where
        P: Params + Send + 'static,
        T: Send + 'static,
        F: FnOnce(&Row<'_>) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        Ok(self
            .client
            .conn(move |conn| conn.prepare_cached(stmt)?.query_row(params, f))
            .await?)
    }

    pub(crate) async fn query_map<T, U, P, F>(
        &self,
        stmt: &'static str,
        params: P,
        func: F,
    ) -> Result<T, DbError>
    where
        T: FromIterator<U> + Send + 'static,
        P: Params + Send + 'static,
        F: FnMut(&Row<'_>) -> Result<U, rusqlite::Error> + Send + 'static,
    {
        Ok(self
            .client
            .conn(move |conn| {
                conn.prepare_cached(stmt)?
                    .query_map(params, func)?
                    .collect()
            })
            .await?)
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Write};

    use types::crypto::Password;

    use crate::statements;

    use super::*;

    #[tokio::test]
    async fn test_set_database_key() {
        File::create("./mock.db").unwrap().write(&[]).unwrap();

        let key = Password::from("SomePasswordtype")
            .derive_new_db_encryption_key()
            .unwrap()
            .0;
        {
            let client = async_sqlite::ClientBuilder::new()
                .path("./mock.db")
                .open()
                .await
                .expect("Failed to open in memory database");

            DataBase::set_database_key(&client, key.clone())
                .await
                .expect("Failed to set database key");

            client
                .conn(|conn| conn.execute(statements::create::CREATE_TABLE_ACCOUNTS, []))
                .await
                .expect("Unable to create table, accounts");
        }
        let second_client = async_sqlite::ClientBuilder::new()
            .path("./mock.db")
            .open()
            .await
            .expect("Failed to open second client");

        let query = second_client
            .conn(|conn| conn.execute(&statements::create::CREATE_TABLE_FUNGIBLE_ASSETS, []))
            .await;
        assert!(query.is_err());

        DataBase::set_database_key(&second_client, key)
            .await
            .expect("Failed to set database key for second client");

        second_client
            .conn(|conn| conn.execute(&statements::create::CREATE_TABLE_NON_FUNGIBLE_ASSETS, []))
            .await
            .expect("Unable to create table, fungibles");
    }
}
