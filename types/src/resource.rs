use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::ResourceAddress;

pub struct Resource {
    pub address: ResourceAddress,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub current_supply: String,
    pub divisibility: Option<u8>,
    pub tags: Tags,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tags(Vec<String>);

impl Deref for Tags {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Tags {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<Vec<String>> for Tags {
    fn into(self) -> Vec<String> {
        self.0
    }
}

impl From<Vec<String>> for Tags {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

impl rusqlite::types::FromSql for Tags {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let blob = value.as_blob()?;

        Ok(serde_json::from_slice(blob)
            .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?)
    }
}

impl rusqlite::types::ToSql for Tags {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let serialized = serde_json::to_string(&self)
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;

        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob(serialized.as_bytes().to_vec()),
        ))
    }
}
