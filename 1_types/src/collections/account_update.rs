use std::collections::{BTreeMap, HashMap};

use crate::{
    Account, Network, Resource,
    address::ResourceAddress,
    assets::{FungibleAsset, NonFungibleAsset},
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty_with_the_given_network() {
        let update = AccountsUpdate::new(Network::Stokenet);
        assert_eq!(update.network, Network::Stokenet);
        assert!(update.account_updates.is_empty());
        assert!(update.new_resources.is_empty());
        assert!(update.icon_urls.is_empty());
    }
}
