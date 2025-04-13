use deps::*;

use std::collections::{BTreeSet, HashMap};

use bytes::Bytes;
use types::{address::{AccountAddress, ResourceAddress}, assets::{FungibleAsset, NonFungibleAsset}, Account, Resource};

#[derive(Debug, Clone)]
pub struct ResourceData {
    pub accounts: HashMap<AccountAddress, Account>,
    pub fungibles: HashMap<AccountAddress, BTreeSet<FungibleAsset>>,
    pub non_fungibles: HashMap<AccountAddress, BTreeSet<NonFungibleAsset>>,
    pub resources: HashMap<ResourceAddress, Resource>,
    pub resource_icons: HashMap<ResourceAddress, Bytes>,
}


impl ResourceData {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
            resources: HashMap::new(),
            resource_icons: HashMap::new(),
        }
    }
}