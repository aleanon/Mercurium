use serde::{Deserialize, Serialize};

use super::explicit_metadata::ExplicitMetadata;

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleResourcesVaultAggregated {
    pub total_count: u32,
    pub items: Vec<NonFungibleResourceVaultAggregated>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleResourceVaultAggregated {
    pub vaults: NFTVaults,
    pub aggregation_level: String,
    pub resource_address: String,
    pub explicit_metadata: super::explicit_metadata::ExplicitMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTVaults {
    pub total_count: u32,
    pub items: Vec<NFTVault>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTVault {
    pub total_count: u32,
    pub items: Vec<String>,
    pub vault_address: String,
    pub last_updated_at_state_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Typed {
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleResourcesGlobalyAggregated {
    pub total_count: u32,
    pub items: Vec<NonFungibleResourceGlobalyAggregated>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleResourceGlobalyAggregated {
    amount: String,
    last_updated_at_state_version: usize,
    aggregation_level: String,
    resource_address: String,
    explicit_metadata: ExplicitMetadata,
}
