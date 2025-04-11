use deps_two::*;

use crate::{
    address::{AccountAddress, Address, ResourceAddress},
    debug_info,
    unwrap_unreachable::UnwrapUnreachable,
};
use async_sqlite::rusqlite;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId([u8; Self::LENGTH]);

impl AssetId {
    pub const LENGTH: usize =
        AccountAddress::CHECKSUM_DOUBLE_LENGTH + ResourceAddress::CHECKSUM_DOUBLE_LENGTH;

    pub fn new(account_address: &AccountAddress, resource_address: &ResourceAddress) -> Self {
        let mut assetid = [0; Self::LENGTH];
        assetid[..AccountAddress::CHECKSUM_DOUBLE_LENGTH]
            .copy_from_slice(account_address.checksum_double_slice());
        assetid[ResourceAddress::CHECKSUM_DOUBLE_LENGTH..]
            .copy_from_slice(resource_address.checksum_double_slice());

        Self(assetid)
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_unreachable(debug_info!("Invalid utf8 in AssetId"))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[cfg(test)]
    pub fn from_array(array: [u8; Self::LENGTH]) -> Self {
        Self(array)
    }
}

impl rusqlite::types::ToSql for AssetId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(&self.0),
        ))
    }
}

impl rusqlite::types::FromSql for AssetId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => Ok(Self(slice.try_into().map_err(|_| {
                rusqlite::types::FromSqlError::InvalidBlobSize {
                    expected_size: Self::LENGTH,
                    blob_size: slice.len(),
                }
            })?)),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

