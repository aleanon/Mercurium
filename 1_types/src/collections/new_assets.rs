use std::collections::{BTreeSet, HashMap};

use crate::address::ResourceAddress;

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
            .extend(other.new_non_fungibles.into_inner());
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
        self.0.extend(other)
    }

    pub fn into_inner(self) -> HashMap<ResourceAddress, Vec<String>> {
        self.0
    }
}
