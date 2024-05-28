use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleResourcesVaultAggregated {
    pub total_count: u32,
    pub items: Vec<FungibleResourceVaultAggregated>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleResourceVaultAggregated {
    pub vaults: Vaults,
    pub aggregation_level: String,
    pub resource_address: String,
    pub explicit_metadata: super::explicit_metadata::ExplicitMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vaults {
    pub total_count: u32,
    pub items: Vec<Vault>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub vault_address: String,
    pub amount: String,
    pub last_updated_at_state_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleResourceGlobalAggregation {
    pub amount: String,
    pub last_updated_at_state_version: usize,
    pub aggregation_level: String,
    pub resource_address: String,
    pub explicit_metadata: super::ExplicitMetadata,
}
