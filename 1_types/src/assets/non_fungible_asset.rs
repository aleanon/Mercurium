use deps::*;

use std::ops::{Deref, DerefMut};

use async_sqlite::rusqlite;
use serde::{Deserialize, Serialize};

use crate::{
    address::{AccountAddress, ResourceAddress},
    response_models::{NFTVaults, non_fungible_id_data::Field},
};

use super::AssetId;

#[derive(Debug, Clone)]
pub struct NonFungibleAsset {
    pub id: AssetId,
    pub resource_address: ResourceAddress,
    pub nfids: NFTs,
}

impl NonFungibleAsset {
    pub fn new(
        account_address: &AccountAddress,
        nfids: NFTs,
        resource_address: ResourceAddress,
    ) -> Self {
        let id = AssetId::new(account_address, &resource_address);
        Self {
            id,
            resource_address,
            nfids,
        }
    }

    pub fn take_nfts(&mut self) -> NFTs {
        std::mem::replace(&mut self.nfids, NFTs::new())
    }

    pub fn nft_id_as_string(&mut self) -> Vec<String> {
        std::mem::replace(&mut self.nfids, NFTs::new())
            .into_iter()
            .map(|nfid| nfid.id)
            .collect()
    }

    #[cfg(test)]
    pub fn placeholder() -> Self {
        Self {
            id: AssetId::from_array([0; AssetId::LENGTH]),
            resource_address: ResourceAddress::empty(crate::Network::Mainnet),
            nfids: NFTs::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTs(Vec<NFT>);

impl NFTs {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn nr_of_nfts(&self) -> usize {
        self.0.len()
    }
}

impl Deref for NFTs {
    type Target = Vec<NFT>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NFTs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for NFTs {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = NFT;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Into<NFT>> FromIterator<T> for NFTs {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(|item| item.into()).collect())
    }
}

impl From<Vec<NFT>> for NFTs {
    fn from(value: Vec<NFT>) -> Self {
        Self(value)
    }
}

impl From<NFTVaults> for NFTs {
    fn from(value: NFTVaults) -> Self {
        Self(
            value
                .items
                .into_iter()
                .flat_map(|vault| {
                    vault.items.into_iter().map(|id| NFT {
                        id,
                        nfdata: Vec::new(),
                    })
                })
                .collect(),
        )
    }
}

impl From<&NFTVaults> for NFTs {
    fn from(value: &NFTVaults) -> Self {
        Self(
            value
                .items
                .iter()
                .flat_map(|vault| {
                    vault.items.iter().map(|id| NFT {
                        id: id.clone(),
                        nfdata: Vec::new(),
                    })
                })
                .collect(),
        )
    }
}

impl rusqlite::types::FromSql for NFTs {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(value) => Ok(serde_json::from_slice(value)
                .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for NFTs {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob(
                serde_json::to_vec(self)
                    .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?,
            ),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub struct NFT {
    pub id: String,
    pub nfdata: Vec<NFData>,
}

impl NFT {
    pub fn new(id: String) -> Self {
        Self {
            id,
            nfdata: Vec::new(),
        }
    }

    pub fn new_with_data(id: String, data: Vec<NFData>) -> Self {
        Self { id, nfdata: data }
    }

    pub fn from_nfid_data(id: String, data: Vec<Field>) -> Self {
        let nfdata = data
            .into_iter()
            .map(|field| NFData {
                key: field.field_name,
                value: field.value,
            })
            .collect();
        Self { id, nfdata }
    }

    pub fn into_id(self) -> String {
        self.id
    }
}

impl PartialEq<String> for NFT {
    fn eq(&self, other: &String) -> bool {
        &self.id == other
    }
}

impl From<String> for NFT {
    fn from(value: String) -> Self {
        Self {
            id: value,
            nfdata: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NFData {
    pub key: String,
    pub value: String,
}
