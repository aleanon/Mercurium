use std::{
    collections::{BTreeSet, HashMap},
    ops::Deref,
};

use crate::{
    debug_info, unwrap_unreachable::UnwrapUnreachable, AccountAddress, NFIDs, ResourceAddress,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId([u8; Self::LENGTH]);

impl AssetId {
    const CHECKSUM_LEN: usize = AccountAddress::CHECKSUM_LENGTH + ResourceAddress::CHECKSUM_LEN;
    const LENGTH: usize = AccountAddress::CHECKSUM_LENGTH + ResourceAddress::CHECKSUM_LEN;

    pub fn new(account_address: &AccountAddress, resource_address: &ResourceAddress) -> Self {
        let mut assetid = [0; Self::LENGTH];
        assetid[..AccountAddress::CHECKSUM_LENGTH]
            .copy_from_slice(account_address.checksum_slice());
        assetid[AccountAddress::CHECKSUM_LENGTH..]
            .copy_from_slice(resource_address.checksum_slice());

        Self(assetid)
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_unreachable(debug_info!("Invalid utf8 in AssetId"))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
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

/// Consists of the symbol for the resource and the account address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FungibleAsset {
    pub id: AssetId,
    pub resource_address: ResourceAddress,
    pub amount: String,
}

impl FungibleAsset {
    pub fn new(
        account_address: &AccountAddress,
        amount: String,
        resource_address: ResourceAddress,
    ) -> Self {
        let id = AssetId::new(account_address, &resource_address);
        Self {
            id,
            amount,
            resource_address,
        }
    }

    pub fn update_with_new_amount(&mut self, amount: String) {
        self.amount = amount;
    }
}
impl PartialOrd for FungibleAsset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.resource_address.cmp(&other.resource_address))
    }
}

impl Ord for FungibleAsset {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.resource_address.cmp(&other.resource_address)
    }
}

#[derive(Debug, Clone)]
pub struct NonFungibleAsset {
    pub id: AssetId,
    pub resource_address: ResourceAddress,
    pub nfids: NFIDs,
}

impl NonFungibleAsset {
    pub fn new(
        account_address: &AccountAddress,
        nfids: NFIDs,
        resource_address: ResourceAddress,
    ) -> Self {
        let id = AssetId::new(account_address, &resource_address);
        Self {
            id,
            resource_address,
            nfids,
        }
    }

    pub fn take_nfids(&mut self) -> NFIDs {
        self.nfids.drain(..).collect()
    }

    pub fn nfids_as_string(&mut self) -> Vec<String> {
        self.nfids.drain(..).map(|nfid| nfid.get_id()).collect()
    }

    pub fn placeholder() -> Self {
        Self {
            id: AssetId([0; AssetId::LENGTH]),
            resource_address: ResourceAddress::empty(),
            nfids: NFIDs::new(),
        }
    }
}

impl PartialEq for NonFungibleAsset {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.resource_address == other.resource_address
    }

    fn ne(&self, other: &Self) -> bool {
        self.id != other.id || self.resource_address != other.resource_address
    }
}

impl Eq for NonFungibleAsset {}

impl PartialOrd for NonFungibleAsset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.resource_address.cmp(&other.resource_address))
    }
}

impl Ord for NonFungibleAsset {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.resource_address.cmp(&other.resource_address)
    }
}

#[derive(Debug)]
pub struct NewAssets {
    pub new_fungibles: BTreeSet<ResourceAddress>,
    pub new_non_fungibles: NewNonFungibles,
}

impl NewAssets {
    pub fn new() -> Self {
        Self {
            new_fungibles: BTreeSet::new(),
            new_non_fungibles: NewNonFungibles::new(),
        }
    }

    pub fn extend(&mut self, other: NewAssets) {
        self.new_fungibles.extend(other.new_fungibles);
        self.new_non_fungibles
            .extend(other.new_non_fungibles.inner());
    }
}

#[derive(Debug)]
pub struct NewNonFungibles(pub HashMap<ResourceAddress, Vec<String>>);

impl NewNonFungibles {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, resource_address: &ResourceAddress, nfid: String) {
        if let Some(nfids) = self.0.get_mut(resource_address) {
            nfids.push(nfid)
        } else {
            self.0.insert(resource_address.clone(), vec![nfid]);
        }
    }

    pub fn extend(&mut self, other: HashMap<ResourceAddress, Vec<String>>) {
        self.0.extend(other.into_iter())
    }

    pub fn inner(self) -> HashMap<ResourceAddress, Vec<String>> {
        self.0
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
                id TEXT NOT NULL PRIMARY KEY,
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
        let assetid = AssetId::new(&account_address, &resource_address);

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
                })
            })
            .unwrap();

        assert_eq!(
            String::from_utf8(fungible_asset.id.0.to_vec()).unwrap(),
            String::from("t2a5axq7fclq")
        );
    }
}
