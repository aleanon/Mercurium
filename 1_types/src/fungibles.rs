use std::{
    collections::BTreeSet,
    ops::{Deref, DerefMut},
};

use super::{Decimal, MetaData, ResourceAddress};

#[derive(Debug, Clone)]
pub struct Fungibles(pub BTreeSet<Fungible>);

impl Fungibles {
    pub fn new() -> Self {
        Self(BTreeSet::new())
    }
    pub fn with_fungibles(fungibles: BTreeSet<Fungible>) -> Self {
        Self(fungibles)
    }
}

// impl From<Vec<Fungible>> for Fungibles {
//     fn from(value: Vec<Fungible>) -> Self {
//         let map = value.into_iter().map(|fungible|
//             (fungible.address.clone(), fungible)
//         )
//         .collect::<HashMap<ResourceAddress, Fungible>>();

//         Self(map)
//     }
// }

// impl From<HashMap<ResourceAddress, Fungible>> for Fungibles {
//     fn from(value: HashMap<ResourceAddress, Fungible>) -> Self {
//         Self(value)
//     }
// }

impl From<BTreeSet<Fungible>> for Fungibles {
    fn from(value: BTreeSet<Fungible>) -> Self {
        Self(value)
    }
}

impl FromIterator<Fungible> for Fungibles {
    fn from_iter<T: IntoIterator<Item = Fungible>>(iter: T) -> Self {
        Self(iter.into_iter().collect::<BTreeSet<Fungible>>())
    }
}

impl<'a> IntoIterator for &'a Fungibles {
    type Item = &'a Fungible;
    type IntoIter = std::collections::btree_set::Iter<'a, Fungible>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Fungibles {
    type Item = Fungible;
    type IntoIter = std::collections::btree_set::IntoIter<Fungible>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for Fungibles {
    type Target = BTreeSet<Fungible>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Fungibles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// impl Iterator for Fungibles {
//     type Item = Fungible;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.next()
//     }

// }

#[derive(Debug, Clone)]
pub struct Fungible {
    pub name: String,
    pub address: ResourceAddress,
    pub symbol: String,
    pub amount: Decimal,
    pub description: Option<String>,
    pub last_updated_at_state_version: i64,
    pub total_supply: String,
    pub metadata: MetaData,
}

impl PartialEq for Fungible {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
    }
    fn ne(&self, other: &Self) -> bool {
        self.symbol != other.symbol
    }
}

impl Eq for Fungible {}

impl PartialOrd for Fungible {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Fungible {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.address.as_str()
            == "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"
        {
            return std::cmp::Ordering::Less;
        } else if other.address.as_str()
            == "resource_rdx1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxradxrd"
        {
            return std::cmp::Ordering::Greater;
        }

        self.symbol.cmp(&other.symbol)
    }
}

// impl From<FungibleResource> for Fungible {
//     fn from(value: FungibleResource) -> Self {
//         let amount: Decimal = value.vaults.items.iter().fold(RadixDecimal::ZERO, |mut acc, vault| {
//             acc.add_assign(RadixDecimal::from_str(&vault.amount).expect("Unable to parse Decimal"));
//             acc
//         }).into();

//         let mut name = None;
//         let mut symbol = None;
//         let icon = None;
//         let mut description = None;

//         let mut metadata = MetaData::new();

//         for item in value.explicit_metadata.items {
//             match &*item.key {
//                 "symbol" => symbol = item.value.typed.value,
//                 "description" => description = item.value.typed.value,
//                 "name" => name = item.value.typed.value,
//                 _ => {metadata.push(item.into())}
//             }
//         };

//         let last_updated_at_state_version:i64 = match value.vaults.items.get(0) {
//             Some(vault) => vault.last_updated_at_state_version as i64,
//             None => 0
//         };

//         Self {
//             name: name.unwrap_or(String::with_capacity(0)),
//             amount,
//             address: ResourceAddress::from_str(&value.resource_address).unwrap(),
//             description,
//             symbol: symbol.unwrap_or(String::with_capacity(0)),
//             icon,
//             current_supply: "Not in use".to_owned(),
//             last_updated_at_state_version,
//             metadata,
//         }

//     }
// }

// impl From<FungibleResources> for Fungibles {
//     fn from(value: FungibleResources) -> Self {
//         Self(
//             value.items.into_iter().map(|fungible|
//                 Fungible::from(fungible)
//             ).collect()
//         )
//     }
// }
