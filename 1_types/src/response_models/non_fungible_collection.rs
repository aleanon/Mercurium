use deps::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleCollectionItemVaultAggregated {
    pub vaults: NonFungibleCollectionItemVaultAggregatedVaults,
    pub resource_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleCollectionItemVaultAggregatedVaults {
    pub total_count: u64,
    pub items: Vec<NonFungibleCollectionItemVaultAggregatedVault>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleCollectionItemVaultAggregatedVault {
    pub total_count: u64,
    pub next_cursor: Option<String>,
    // pub items: Vec<String>,
    pub vault_address: String,
    pub last_updated_at_state_version: i64,
}
