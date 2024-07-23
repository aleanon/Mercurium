use std::collections::{BTreeMap, HashMap};

use crate::{
    assets::{FungibleAsset, NonFungibleAsset},
    Account, Network, Resource, ResourceAddress,
};

#[derive(Debug, Clone)]
pub struct AccountUpdate {
    pub account: Account,
    pub fungibles: HashMap<ResourceAddress, FungibleAsset>,
    pub non_fungibles: HashMap<ResourceAddress, NonFungibleAsset>,
}

#[derive(Debug, Clone)]
pub struct AccountsUpdate {
    pub network: Network,
    pub account_updates: Vec<AccountUpdate>,
    pub new_resources: HashMap<ResourceAddress, Resource>,
    pub icon_urls: BTreeMap<ResourceAddress, String>,
}

impl AccountsUpdate {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            account_updates: Vec::new(),
            new_resources: HashMap::new(),
            icon_urls: BTreeMap::new(),
        }
    }
}
