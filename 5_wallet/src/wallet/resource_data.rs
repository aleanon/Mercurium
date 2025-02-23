use std::collections::{BTreeSet, HashMap};

use types::{address::{AccountAddress, ResourceAddress}, assets::{FungibleAsset, NonFungibleAsset}, Account, Resource};


pub struct ResourceData {
    accounts: HashMap<AccountAddress, Account>,
    fungibles: HashMap<AccountAddress, BTreeSet<FungibleAsset>>,
    non_fungibles: HashMap<AccountAddress, BTreeSet<NonFungibleAsset>>,
    resources: HashMap<ResourceAddress, Resource>,
}


impl ResourceData {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
            resources: HashMap::new(),
        }
    }
}