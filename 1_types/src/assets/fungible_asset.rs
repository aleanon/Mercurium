use std::cmp::Ordering;

use deps::*;

use serde::{Deserialize, Serialize};

use crate::address::{AccountAddress, ResourceAddress, XRD};

use super::AssetId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FungibleAsset {
    pub id: AssetId,
    pub resource_address: ResourceAddress,
    pub amount: String,
}

impl FungibleAsset {
    pub fn new(
        account_address: &AccountAddress,
        amount: String,
        resource_address: ResourceAddress,
    ) -> Self {
        let id = AssetId::new(account_address, &resource_address);
        Self {
            id,
            amount,
            resource_address,
        }
    }

    pub fn update_with_new_amount(&mut self, amount: String) {
        self.amount = amount;
    }
}
impl PartialOrd for FungibleAsset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.resource_address.cmp(&other.resource_address))
    }
}

impl Ord for FungibleAsset {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let ResourceAddress::Mainnet(addr) = other.resource_address {
            if &addr == XRD {
                return Ordering::Less;
            }
        }
        self.resource_address.cmp(&other.resource_address)
    }
}
