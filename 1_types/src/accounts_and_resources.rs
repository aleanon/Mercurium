use std::collections::{BTreeSet, HashMap};

use crate::{
    assets::{FungibleAsset, NonFungibleAsset},
    Account, AccountAddress, Resource, ResourceAddress,
};

#[derive(Debug, Clone)]
pub struct AppdataFromDisk {
    pub accounts: HashMap<AccountAddress, Account>,
    pub fungible_assets: HashMap<AccountAddress, BTreeSet<FungibleAsset>>,
    pub non_fungible_assets: HashMap<AccountAddress, BTreeSet<NonFungibleAsset>>,
    pub resources: HashMap<ResourceAddress, Resource>,
}
