use deps::*;

use std::{fmt::Debug, num::NonZeroU32, path::Path};

use async_sqlite::rusqlite::{
    self, CachedStatement, Connection, ErrorCode, Params, Result, Row, ffi,
};
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
    #[error("Unsupported database schema version {found} (this build expects {expected})")]
    UnsupportedSchemaVersion { found: i64, expected: i64 },
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

    /// Current schema version stamped into `PRAGMA user_version`. Bump when the
    /// schema changes and add a corresponding arm to [`Self::migrate`].
    pub(crate) const SCHEMA_VERSION: i64 = 1;

    /// Reconciles the database's stored `user_version` with [`Self::SCHEMA_VERSION`].
    /// Call after the tables are (re)created.
    ///
    /// - `0` — a fresh or pre-versioning database: stamp it with the current version.
    /// - equal — nothing to do.
    /// - older/newer — hand off to [`Self::migrate`]. There is no released schema to
    ///   migrate from yet (pre-1.0), so any real mismatch is currently an error.
    pub(crate) async fn ensure_schema_version(&self) -> Result<(), DbError> {
        let found = self
            .conn(|conn| conn.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0)))
            .await?;

        if found == 0 {
            return self
                .conn(|conn| conn.pragma_update(None, "user_version", Self::SCHEMA_VERSION))
                .await;
        }
        if found == Self::SCHEMA_VERSION {
            return Ok(());
        }
        self.migrate(found).await
    }

    /// Schema-migration seam. Intentionally empty: there is no released schema
    /// version to migrate from, so every non-current version is rejected rather
    /// than silently mis-read. The first post-1.0 schema change adds arms here.
    async fn migrate(&self, found: i64) -> Result<(), DbError> {
        Err(DbError::UnsupportedSchemaVersion {
            found,
            expected: Self::SCHEMA_VERSION,
        })
    }

    pub(crate) async fn conn<T, F>(&self, f: F) -> Result<T, DbError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client
            .conn(f)
            .await
            .map_err(DbError::AsyncSqliteError)
    }

    pub(crate) async fn conn_mut<T, F>(&self, f: F) -> Result<T, DbError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client
            .conn_mut(f)
            .await
            .map_err(DbError::AsyncSqliteError)
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
            .map_err(DbError::AsyncSqliteError)
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
            .map_err(DbError::AsyncSqliteError)
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
    use std::fs::File;

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
        File::create("./mock.db").unwrap();

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

    async fn in_memory_db() -> DataBase {
        let client = async_sqlite::ClientBuilder::new().open().await.unwrap();
        DataBase { client }
    }

    async fn user_version(db: &DataBase) -> i64 {
        db.conn(|conn| conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0)))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn ensure_schema_version_stamps_fresh_db_and_is_idempotent() {
        let db = in_memory_db().await;
        assert_eq!(user_version(&db).await, 0, "fresh db starts at 0");

        db.ensure_schema_version().await.unwrap();
        assert_eq!(user_version(&db).await, DataBase::SCHEMA_VERSION);

        // Second call on an already-current db is a no-op, not an error.
        db.ensure_schema_version().await.unwrap();
        assert_eq!(user_version(&db).await, DataBase::SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn ensure_schema_version_rejects_unknown_version() {
        let db = in_memory_db().await;
        db.conn(|conn| conn.pragma_update(None, "user_version", 999_i64))
            .await
            .unwrap();

        let err = db.ensure_schema_version().await.unwrap_err();
        assert!(
            matches!(err, DbError::UnsupportedSchemaVersion { found: 999, .. }),
            "got {err:?}"
        );
    }
}

// pub struct SyncDataBase {
//     pub(crate) client: rusqlite::Connection,
// }

// impl SyncDataBase {
//     pub(crate) fn load(path: &Path, key: Key<DataBase>) -> Result<Self, DbError> {
//         let client = rusqlite::Connection::open(path)?;
//         client.pragma_update(None, "key", SqliteKey::from_key(&key))?;

//         Ok(Self { client })
//     }

//     pub(crate) fn transaction<F>(
//         &mut self,
//         stmt: &'static str,
//         execute_stmt: F,
//     ) -> Result<(), DbError>
//     where
//         F: FnOnce(&mut CachedStatement) -> Result<(), rusqlite::Error>,
//     {
//         let tx = self.client.transaction()?;

//         execute_stmt(&mut tx.prepare_cached(stmt)?)?;

//         Ok(tx.commit()?)
//     }

//     pub(crate) fn query_row<T, P, F>(
//         &self,
//         stmt: &'static str,
//         params: P,
//         f: F,
//     ) -> Result<T, DbError>
//     where
//         P: Params,
//         F: FnOnce(&Row<'_>) -> Result<T, rusqlite::Error>,
//     {
//         Ok(self.client.prepare_cached(stmt)?.query_row(params, f)?)
//     }

//     pub(crate) fn query_map<T, U, P, F>(
//         &self,
//         stmt: &'static str,
//         params: P,
//         func: F,
//     ) -> Result<T, DbError>
//     where
//         T: FromIterator<U>,
//         P: Params,
//         F: FnMut(&Row<'_>) -> Result<U, rusqlite::Error>,
//     {
//         Ok(self
//             .client
//             .prepare_cached(stmt)?
//             .query_map(params, func)?
//             .flatten()
//             .collect())
//     }
// }
