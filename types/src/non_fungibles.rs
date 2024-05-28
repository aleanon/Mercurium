use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    ops::{Deref, DerefMut},
};

use crate::response_models::entity_details::NFTVaults;

use super::{Icon, MetaData, ResourceAddress};

#[derive(Debug, Clone)]
pub struct NonFungibles(pub BTreeSet<NonFungible>);

impl NonFungibles {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
    pub fn push(&mut self, item: NonFungible) {
        self.0.insert(item);
    }

    // pub fn as_slice(&self) -> &[NonFungible] {
    //     &self.0.
    // }
}

impl From<BTreeSet<NonFungible>> for NonFungibles {
    fn from(value: BTreeSet<NonFungible>) -> Self {
        Self(value)
    }
}

impl FromIterator<NonFungible> for NonFungibles {
    fn from_iter<T: IntoIterator<Item = NonFungible>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<BTreeSet<NonFungible>>())
    }
}

impl<'a> IntoIterator for &'a NonFungibles {
    type Item = &'a NonFungible;
    type IntoIter = std::collections::btree_set::Iter<'a, NonFungible>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Vec<NonFungible>> for NonFungibles {
    fn from(value: Vec<NonFungible>) -> Self {
        value.into_iter().collect()
    }
}

impl Deref for NonFungibles {
    type Target = BTreeSet<NonFungible>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NonFungibles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct NonFungible {
    pub name: String,
    pub symbol: String,
    pub icon: Option<Icon>,
    pub description: Option<String>,
    pub nfids: NFIDs,
    pub address: ResourceAddress,
    pub last_updated_at_state_version: i64,
    pub metadata: MetaData,
}

impl PartialEq for NonFungible {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Eq for NonFungible {}

impl PartialOrd for NonFungible {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NonFungible {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.symbol.cmp(&other.symbol)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFIDs(BTreeSet<NFID>);

impl NFIDs {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub fn nr_of_nfts(&self) -> usize {
        self.0.len()
    }
}

impl Deref for NFIDs {
    type Target = BTreeSet<NFID>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NFIDs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<NFTVaults> for NFIDs {
    fn from(value: NFTVaults) -> Self {
        Self(
            value
                .items
                .into_iter()
                .flat_map(|vault| {
                    vault.items.into_iter().map(|id| NFID {
                        id,
                        nfdata: Vec::new(),
                    })
                })
                .collect(),
        )
    }
}

impl From<&NFTVaults> for NFIDs {
    fn from(value: &NFTVaults) -> Self {
        Self(
            value
                .items
                .iter()
                .flat_map(|vault| {
                    vault.items.iter().map(|id| NFID {
                        id: id.clone(),
                        nfdata: Vec::new(),
                    })
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Ord, Eq)]
pub struct NFID {
    id: String,
    nfdata: Vec<NFData>,
}

impl NFID {
    pub fn new(id: String) -> Self {
        Self {
            id,
            nfdata: Vec::new(),
        }
    }

    pub fn get_id(self) -> String {
        self.id
    }
}

impl PartialEq<String> for NFID {
    fn eq(&self, other: &String) -> bool {
        &self.id == other
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NFData {
    key: String,
    value: String,
}

impl rusqlite::types::FromSql for NFIDs {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(value) => Ok(serde_json::from_slice(value)
                .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for NFIDs {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob(
                serde_json::to_vec(self)
                    .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?,
            ),
        ))
    }
}
