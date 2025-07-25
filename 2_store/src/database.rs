use deps::*;

use std::{fmt::Debug, num::NonZeroU32, ops::Deref, path::Path};

use async_sqlite::rusqlite::{
    self, CachedStatement, Connection, ErrorCode, Params, Result, Row, ffi,
};
use debug_print::debug_println;
use thiserror::Error;
use types::{
    AppPathError,
    crypto::{Key, KeyType},
};

use crate::sqlite_key::SqliteKey;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("{0}")]
    AsyncSqliteError(async_sqlite::Error),
    #[error("Wrong password")]
    IncorrectKey,
    #[error("Database not loaded")]
    DatabaseNotLoaded,
    #[error("Database not found")]
    DatabaseNotFound,
    #[error("Unable to establish path {0}")]
    PathError(#[from] AppPathError),
}

impl From<rusqlite::Error> for DbError {
    fn from(value: rusqlite::Error) -> Self {
        Self::AsyncSqliteError(async_sqlite::Error::Rusqlite(value))
    }
}

impl From<async_sqlite::Error> for DbError {
    fn from(value: async_sqlite::Error) -> Self {
        match value {
            async_sqlite::Error::Rusqlite(rusqlite::Error::SqliteFailure(
                ffi::Error {
                    code: ErrorCode::NotADatabase,
                    extended_code: _,
                },
                None,
            )) => Self::IncorrectKey,
            _ => Self::AsyncSqliteError(value),
        }
    }
}

#[derive(Clone)]
pub struct DataBase {
    pub(crate) client: async_sqlite::Client,
}

impl DataBase {
    pub(crate) async fn load(path: &Path, key: Key<DataBase>) -> Result<Self, DbError> {
        let db = Self::new_with_async_client(path).await?;
        db.set_database_key(key).await?;

        Ok(db)
    }

    async fn new_with_async_client(path: &Path) -> Result<Self, async_sqlite::Error> {
        let client = async_sqlite::ClientBuilder::new().path(path).open().await?;

        Ok(Self { client })
    }

    async fn set_database_key(&self, key: Key<DataBase>) -> Result<(), DbError> {
        self.conn(move |conn| conn.pragma_update(None, "key", SqliteKey::from_key(&key)))
            .await
    }

    pub(crate) async fn conn<T, F>(&self, f: F) -> Result<T, DbError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client
            .conn(f)
            .await
            .map_err(|err| DbError::AsyncSqliteError(err))
    }

    pub(crate) async fn conn_mut<T, F>(&self, f: F) -> Result<T, DbError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client
            .conn_mut(f)
            .await
            .map_err(|err| DbError::AsyncSqliteError(err))
    }

    pub(crate) async fn execute_batch(&self, stmt: &'static str) -> Result<(), DbError> {
        self.conn(move |conn| conn.execute_batch(stmt)).await
    }

    pub(crate) async fn transaction<F>(
        &self,
        stmt: &'static str,
        execute_stmt: F,
    ) -> Result<(), DbError>
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
    ) -> Result<T, DbError>
    where
        F: FnOnce(&mut CachedStatement<'_>) -> Result<T, rusqlite::Error> + Send + 'static,
        T: Send + 'static,
    {
        self.conn_mut(|conn| {
            let mut cached_statement = conn.prepare_cached(stmt)?;
            func(&mut cached_statement)
        })
        .await
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
        self.client
            .conn(move |conn| conn.prepare_cached(stmt)?.query_row(params, f))
            .await
            .map_err(|err| DbError::AsyncSqliteError(err))
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
        self.client
            .conn(move |conn| {
                conn.prepare_cached(stmt)?
                    .query_map(params, func)?
                    .collect()
            })
            .await
            .map_err(|err| DbError::AsyncSqliteError(err))
    }
}

impl Debug for DataBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataBase")
    }
}

impl KeyType for DataBase {
    const KEY_LENGTH: usize = 32;
    const ITERATIONS: std::num::NonZeroU32 = NonZeroU32::new(200000).unwrap();
}

#[cfg(test)]
pub mod test {
    use std::{fs::File, io::Write};

    use types::crypto::{KeySaltPair, Password};

    use crate::app_data_db::statements::{accounts, fungible_assets, non_fungible_assets};

    use super::*;

    pub fn execute_stmt(stmt: &str) -> Result<(), async_sqlite::Error> {
        let client = async_sqlite::ClientBuilder::new().open_blocking().unwrap();
        let stmt = stmt.to_owned();
        client
            .conn_blocking(move |conn| conn.execute(stmt.as_str(), []))
            .map(|_| ())
    }

    pub fn execute_batch_stmt(stmt: &str) -> Result<(), async_sqlite::Error> {
        let client = async_sqlite::ClientBuilder::new().open_blocking().unwrap();
        let stmt = stmt.to_owned();
        client
            .conn_blocking(move |conn| conn.execute_batch(stmt.as_str()))
            .map(|_| ())
    }

    #[tokio::test]
    async fn test_set_database_key() {
        File::create("./mock.db").unwrap().write(&[]).unwrap();

        let key = KeySaltPair::new(Password::from("SomePasswordtype").as_str())
            .unwrap()
            .into_key();
        {
            let client = async_sqlite::ClientBuilder::new()
                .path("./mock.db")
                .open()
                .await
                .expect("Failed to open in memory database");
            let db = DataBase { client };

            db.set_database_key(key.clone())
                .await
                .expect("Failed to set database key");

            db.conn(|conn| conn.execute(accounts::CREATE_TABLE_ACCOUNTS, []))
                .await
                .expect("Unable to create table, accounts");
        }
        let second_client = async_sqlite::ClientBuilder::new()
            .path("./mock.db")
            .open()
            .await
            .expect("Failed to open second client");

        let query = second_client
            .conn(|conn| conn.execute(fungible_assets::CREATE_TABLE_FUNGIBLE_ASSETS, []))
            .await;
        assert!(query.is_err());

        let db = DataBase {
            client: second_client,
        };

        db.set_database_key(key)
            .await
            .expect("Failed to set database key for second client");

        db.conn(|conn| conn.execute(non_fungible_assets::CREATE_TABLE_NON_FUNGIBLE_ASSETS, []))
            .await
            .expect("Unable to create table, fungibles");
    }
}

pub struct SyncDataBase {
    pub(crate) client: rusqlite::Connection,
}

impl SyncDataBase {
    pub(crate) fn load(path: &Path, key: Key<DataBase>) -> Result<Self, DbError> {
        let client = rusqlite::Connection::open(path)?;
        client.pragma_update(None, "key", SqliteKey::from_key(&key))?;

        Ok(Self { client })
    }

    pub(crate) fn transaction<F>(
        &mut self,
        stmt: &'static str,
        execute_stmt: F,
    ) -> Result<(), DbError>
    where
        F: FnOnce(&mut CachedStatement) -> Result<(), rusqlite::Error>,
    {
        let tx = self.client.transaction()?;

        execute_stmt(&mut tx.prepare_cached(stmt)?)?;

        Ok(tx.commit()?)
    }

    pub(crate) fn query_row<T, P, F>(
        &self,
        stmt: &'static str,
        params: P,
        f: F,
    ) -> Result<T, DbError>
    where
        P: Params,
        F: FnOnce(&Row<'_>) -> Result<T, rusqlite::Error>,
    {
        Ok(self.client.prepare_cached(stmt)?.query_row(params, f)?)
    }

    pub(crate) fn query_map<T, U, P, F>(
        &self,
        stmt: &'static str,
        params: P,
        func: F,
    ) -> Result<T, DbError>
    where
        T: FromIterator<U>,
        P: Params,
        F: FnMut(&Row<'_>) -> Result<U, rusqlite::Error>,
    {
        Ok(self
            .client
            .prepare_cached(stmt)?
            .query_map(params, func)?
            .flatten()
            .collect())
    }
}
