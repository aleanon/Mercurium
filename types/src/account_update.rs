use std::collections::{BTreeMap, HashMap};

use crate::{
    assets::{FungibleAsset, NonFungibleAsset},
    Account, Resource, ResourceAddress,
};

#[derive(Debug, Clone)]
pub struct AccountUpdate {
    pub account: Account,
    pub fungibles: HashMap<ResourceAddress, FungibleAsset>,
    pub non_fungibles: HashMap<ResourceAddress, NonFungibleAsset>,
}

#[derive(Debug, Clone)]
pub struct AccountsUpdate {
    pub account_updates: Vec<AccountUpdate>,
    pub new_resources: HashMap<ResourceAddress, Resource>,
    pub icon_urls: BTreeMap<ResourceAddress, String>,
}

impl AccountsUpdate {
    pub fn new() -> Self {
        Self {
            account_updates: Vec::new(),
            new_resources: HashMap::new(),
            icon_urls: BTreeMap::new(),
        }
    }
}
