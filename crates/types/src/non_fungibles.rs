use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, ops::{Deref, DerefMut}};

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
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NonFungible {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.symbol.cmp(&other.symbol)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFIDs(Vec<NFID>);

impl NFIDs {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(&mut self, nfid: NFID) {
        self.0.push(nfid)
    }
    pub fn nr_of_nfts(&self) -> usize {
        self.0.len()
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
                        description: None,
                        icon: None,
                        nfdata: None,
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
                        description: None,
                        icon: None,
                        nfdata: None,
                    })
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFID {
    id: String,
    icon: Option<Icon>,
    description: Option<String>,
    nfdata: Option<Vec<NFData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFData {
    key: String,
    value: String,
}

// impl From<NonFungibleResource> for NonFungible {
//     fn from(value: NonFungibleResource) -> Self {
//         let mut name = None;
//         let mut symbol = String::with_capacity(0);
//         let icon = None;
//         let mut description = None;
//         let mut metadata = MetaData::new();
//         let last_updated_at_state_version = match value.vaults.items.get(0) {
//             Some(vault) => vault.last_updated_at_state_version as i64,
//             None => 0,
//         };

//         for item in value.explicit_metadata.items {
//             match &*item.key {
//                 "name" => name = item.value.typed.value,
//                 "symbol" => symbol = item.value.typed.value.unwrap_or(String::with_capacity(0)),
//                 "description" => description = item.value.typed.value,
//                 _ => metadata.push(item.into()),
//             }
//         }

//         Self {
//             nfids: NFIDs::from(value.vaults),
//             address: ResourceAddress::from_str(&value.resource_address).unwrap(),
//             metadata,
//             name,
//             symbol,
//             icon,
//             last_updated_at_state_version,
//             description,
//         }
//     }
// }

// impl From<NonFungibleResources> for NonFungibles {
//     fn from(value: NonFungibleResources) -> Self {
//         Self(value.items.into_iter().map(|nfts| nfts.into()).collect())
//     }
// }

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
