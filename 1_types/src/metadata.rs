use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

use super::response_models::entity_details::{self, ExplicitMetadata};

///Collection of `MetaDataItems`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaData(Vec<MetaDataItem>);

impl MetaData {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(&mut self, item: MetaDataItem) {
        self.0.push(item)
    }
}

impl Deref for MetaData {
    type Target = Vec<MetaDataItem>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MetaData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ExplicitMetadata> for MetaData {
    fn from(value: ExplicitMetadata) -> Self {
        Self(
            value
                .items
                .into_iter()
                .map(|metadataitem| MetaDataItem {
                    key: metadataitem.key,
                    value: metadataitem.value.typed.value,
                    is_locked: metadataitem.is_locked,
                })
                .collect(),
        )
    }
}

impl rusqlite::types::FromSql for MetaData {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(value) => Ok(serde_json::from_slice(value)
                .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for MetaData {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob(
                serde_json::to_vec(self)
                    .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?,
            ),
        ))
    }
}

///Key-value pair for storing meta-data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaDataItem {
    pub key: String,
    pub value: Option<String>,
    pub is_locked: bool,
}
impl From<entity_details::MetadataItem> for MetaDataItem {
    fn from(value: entity_details::MetadataItem) -> Self {
        MetaDataItem {
            key: value.key,
            value: value.value.typed.value,
            is_locked: value.is_locked,
        }
    }
}
