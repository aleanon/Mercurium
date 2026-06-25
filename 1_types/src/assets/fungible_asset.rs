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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const ACCOUNT: &str =
        "account_tdx_2_12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66";
    const RESOURCE: &str =
        "resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc";

    fn asset(amount: &str) -> FungibleAsset {
        FungibleAsset::new(
            &AccountAddress::from_str(ACCOUNT).unwrap(),
            amount.to_string(),
            ResourceAddress::from_str(RESOURCE).unwrap(),
        )
    }

    #[test]
    fn new_sets_amount_and_resource() {
        let a = asset("100");
        assert_eq!(a.amount, "100");
        assert_eq!(a.resource_address, ResourceAddress::from_str(RESOURCE).unwrap());
    }

    #[test]
    fn update_changes_amount_only() {
        let mut a = asset("100");
        let resource = a.resource_address.clone();
        a.update_with_new_amount("250".to_string());
        assert_eq!(a.amount, "250");
        assert_eq!(a.resource_address, resource);
    }

    #[test]
    fn equality_compares_all_fields() {
        // Same account + resource + amount are equal; a different amount is not.
        // (Full serde round-trips aren't valid here: AssetId deserializes from borrowed
        // SQLite blob bytes, not from owned serde_json input.)
        assert_eq!(asset("5"), asset("5"));
        assert_ne!(asset("5"), asset("6"));
    }
}
