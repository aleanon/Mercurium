use std::{num::NonZeroU32, path::Path};

use async_trait::async_trait;
use deps::{
    async_sqlite::{
        self,
        rusqlite::{self, CachedStatement, Connection, Params, Row},
    },
    zeroize::ZeroizeOnDrop,
};
use types::{
    UnwrapUnreachable,
    crypto::{Key, KeyType},
    debug_info,
};

use crate::ports::data_store::{DataStore, DataStoreError};

impl From<async_sqlite::Error> for DataStoreError {
    fn from(err: async_sqlite::Error) -> Self {
        DataStoreError::RepositoryError(Box::new(err))
    }
}

#[derive(Clone)]
pub struct Sqlite {
    pub(crate) client: async_sqlite::Client,
}

impl Sqlite {
    pub(crate) async fn load(path: &Path, key: Key<Sqlite>) -> Result<Self, DataStoreError> {
        let db = Self::new_with_async_client(path).await?;
        db.set_database_key(key).await?;

        Ok(db)
    }

    async fn new_with_async_client(path: &Path) -> Result<Self, async_sqlite::Error> {
        let client = async_sqlite::ClientBuilder::new().path(path).open().await?;

        Ok(Self { client })
    }

    async fn set_database_key(&self, key: Key<Sqlite>) -> Result<(), DataStoreError> {
        self.conn(move |conn| conn.pragma_update(None, "key", HexKey::from_key(&key)))
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn conn<T, F>(&self, f: F) -> Result<T, DataStoreError>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client.conn(f).await.map_err(Into::into)
    }

    pub(crate) async fn conn_mut<T, F>(&self, f: F) -> Result<T, DataStoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client.conn_mut(f).await.map_err(Into::into)
    }

    pub(crate) async fn execute_batch(&self, stmt: &'static str) -> Result<(), DataStoreError> {
        self.conn(move |conn| conn.execute_batch(stmt)).await
    }

    pub(crate) async fn transaction<F>(
        &self,
        stmt: &'static str,
        execute_stmt: F,
    ) -> Result<(), DataStoreError>
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
    ) -> Result<T, DataStoreError>
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
    ) -> Result<T, DataStoreError>
    where
        P: Params + Send + 'static,
        T: Send + 'static,
        F: FnOnce(&Row<'_>) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        self.client
            .conn(move |conn| conn.prepare_cached(stmt)?.query_row(params, f))
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn query_map<T, U, P, F>(
        &self,
        stmt: &'static str,
        params: P,
        func: F,
    ) -> Result<T, DataStoreError>
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
            .map_err(Into::into)
    }
}

impl KeyType for Sqlite {
    const ITERATIONS: std::num::NonZeroU32 = NonZeroU32::new(200000).unwrap();
    const KEY_LENGTH: usize = 32;
}

#[derive(Debug, ZeroizeOnDrop, Clone)]
struct HexKey(Vec<u8>);

impl HexKey {
    const KEY_START: &[u8] = b"x\'";
    const KEY_END: u8 = b'\'';

    pub fn from_key(key: &Key<Sqlite>) -> Self {
        let key_length = key.as_bytes().len() * 2;
        let mut db_hex_key = vec![0u8; key_length];

        for (i, byte) in key.as_bytes().iter().enumerate() {
            db_hex_key[i * 2] = Self::to_hex_digit(byte >> 4);
            db_hex_key[i * 2 + 1] = Self::to_hex_digit(byte & 0x0F);
        }

        db_hex_key.insert(0, Self::KEY_START[1]);
        db_hex_key.insert(0, Self::KEY_START[0]);
        db_hex_key.push(Self::KEY_END);

        Self(db_hex_key)
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0)
            .unwrap_unreachable(debug_info!("HexKey contained non utf8 bytes"))
    }

    fn to_hex_digit(n: u8) -> u8 {
        match n {
            0..=9 => b'0' + n,
            10..=15 => b'a' + (n - 10),
            _ => unreachable!(),
        }
    }
}

impl async_sqlite::rusqlite::ToSql for HexKey {
    fn to_sql(
        &self,
    ) -> Result<async_sqlite::rusqlite::types::ToSqlOutput, async_sqlite::rusqlite::Error> {
        Ok(async_sqlite::rusqlite::types::ToSqlOutput::Borrowed(
            async_sqlite::rusqlite::types::ValueRef::Text(&self.0),
        ))
    }
}

#[async_trait]
impl DataStore for Sqlite {
    async fn init_repository<P: AsRef<Path> + Send>(
        path: P,
        key: types::crypto::Key<Self>,
    ) -> Result<Self, DataStoreError> {
        let client = async_sqlite::ClientBuilder::new()
            .path(path.as_ref())
            .open()
            .await
            .map_err(|_| DataStoreError::UnableToCreateRepository)?;

        let db = Self { client };
        db.set_database_key(key).await?;
        Ok(db)
    }

    async fn load<P: AsRef<Path> + Send>(
        path: P,
        key: types::crypto::Key<Self>,
    ) -> Result<Self, DataStoreError> {
        let client = async_sqlite::ClientBuilder::new()
            .path(path.as_ref())
            .open()
            .await
            .map_err(|_| DataStoreError::UnableToLoadRepository)?;

        let db = Self { client };
        db.set_database_key(key).await?;
        Ok(db)
    }

    async fn delete_repository<P: AsRef<Path> + Send>(
        &self,
        path: P,
    ) -> Result<(), DataStoreError> {
        self.client
            .close()
            .await
            .map_err(|e| DataStoreError::RepositoryError(Box::new(e)))?;

        std::fs::remove_file(path).map_err(|_| DataStoreError::UnableToDeleteRepository)?;
        Ok(())
    }
}
