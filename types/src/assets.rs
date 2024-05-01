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
        let account_checksum = account_address.checksum();
        let resource_checksum = resource_address.checksum();
        let mut checksum = [0u8; Self::COMBINED_CHECKSUM_LEN];
        checksum[..Self::HALF_CHECKSUM_LEN].copy_from_slice(&account_checksum);
        checksum[Self::HALF_CHECKSUM_LEN..].copy_from_slice(&resource_checksum);
        Self(symbol, checksum)
    }
}

impl rusqlite::types::ToSql for AssetId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::from(
            [self.0.as_bytes(), self.1.as_slice()].concat(),
        ))
    }
}

impl rusqlite::types::FromSql for AssetId {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => {
                let end = slice.len();
                let (symbol, checksum) = slice.split_at(end - Self::COMBINED_CHECKSUM_LEN);
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
