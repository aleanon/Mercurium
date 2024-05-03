use crate::{AccountAddress, NFIDs, ResourceAddress};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(String, [u8; Self::COMBINED_CHECKSUM_LEN]);

impl AssetId {
    const COMBINED_CHECKSUM_LEN: usize =
        AccountAddress::CHECKSUM_LEN + ResourceAddress::CHECKSUM_LEN;
    const HALF_CHECKSUM_LEN: usize = Self::COMBINED_CHECKSUM_LEN / 2;

    pub fn new(
        symbol: String,
        account_address: &AccountAddress,
        resource_address: &ResourceAddress,
    ) -> Self {
        let mut checksum = [0u8; Self::COMBINED_CHECKSUM_LEN];
        checksum[..Self::HALF_CHECKSUM_LEN].copy_from_slice(&account_address.checksum());
        checksum[Self::HALF_CHECKSUM_LEN..].copy_from_slice(&resource_address.checksum());
        Self(symbol, checksum)
    }
}

impl rusqlite::types::ToSql for AssetId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob([self.0.as_bytes(), self.1.as_slice()].concat()),
        ))
    }
}

impl rusqlite::types::FromSql for AssetId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => {
                let (symbol, checksum) = slice.split_at(slice.len() - Self::COMBINED_CHECKSUM_LEN);
                let symbol = String::from_utf8(symbol.to_owned())
                    .map_err(|_| rusqlite::types::FromSqlError::InvalidType)?;

                let checksum: [u8; Self::COMBINED_CHECKSUM_LEN] =
                    checksum.try_into().map_err(|_| {
                        rusqlite::types::FromSqlError::InvalidBlobSize {
                            expected_size: Self::COMBINED_CHECKSUM_LEN,
                            blob_size: checksum.len(),
                        }
                    })?;

                Ok(Self(symbol, checksum))
            }
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

/// Consists of the symbol for the resource and the account address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FungibleAsset {
    pub id: AssetId,
    pub resource_address: ResourceAddress,
    pub amount: String,
    pub last_updated: usize,
}

impl FungibleAsset {
    pub fn new(
        symbol: String,
        account_address: &AccountAddress,
        amount: String,
        resource_address: ResourceAddress,
        last_updated: usize,
    ) -> Self {
        let id = AssetId::new(symbol, account_address, &resource_address);
        Self {
            id,
            amount,
            resource_address,
            last_updated,
        }
    }

    pub fn update_with_new_amount(&mut self, amount: String, state_version: usize) {
        self.amount = amount;
        self.last_updated = state_version;
    }
}

pub struct NonFungibleAsset {
    pub id: AssetId,
    pub resource_address: ResourceAddress,
    pub nfids: NFIDs,
    pub last_updated: usize,
}

impl NonFungibleAsset {
    pub fn new(
        symbol: String,
        account_address: &AccountAddress,
        nfids: NFIDs,
        resource_address: ResourceAddress,
        last_updated: usize,
    ) -> Self {
        let id = AssetId::new(symbol, account_address, &resource_address);
        Self {
            id,
            resource_address,
            nfids,
            last_updated,
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use rusqlite::params;

    use super::*;
    #[test]
    fn test_assetid_serialization() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();

        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS fungible_assets (
                id BLOB NOT NULL PRIMARY KEY,
                resource_address BLOB NOT NULL,
                amount TEXT NOT NULL,
                last_updated INTEGER NOT NULL)",
                [],
            )
            .unwrap();

        let account_address = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .unwrap();
        let resource_address = ResourceAddress::from_str(
            "resource_rdx1t5ywq4c6nd2lxkemkv4uzt8v7x7smjcguzq5sgafwtasa6luq7fclq",
        )
        .unwrap();
        let assetid = AssetId::new("GUM".to_string(), &account_address, &resource_address);

        connection
            .execute(
                "INSERT INTO
            fungible_assets (
                id,
                resource_address,
                amount,
                last_updated
            )
            VALUES (?, ?, ?, ?)
        ",
                params![assetid, resource_address, "10", 1 as i64,],
            )
            .unwrap();

        let mut prepared = connection
            .prepare("SELECT * FROM fungible_assets WHERE id = ?")
            .unwrap();
        let fungible_asset = prepared
            .query_row(params![assetid], |row| {
                Ok(FungibleAsset {
                    id: row.get(0)?,
                    resource_address: row.get(1)?,
                    amount: row.get(2)?,
                    last_updated: row.get(3)?,
                })
            })
            .unwrap();

        assert_eq!(fungible_asset.id.0, "GUM".to_string());
        assert_eq!(
            std::str::from_utf8(fungible_asset.id.1.as_slice()).unwrap(),
            "t2a5axq7fclq"
        );
    }
}
