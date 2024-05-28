mod details;
mod explicit_metadata;
mod fungible_resources;
mod metadata;
mod non_fungible_resources;

pub use fungible_resources::FungibleResourceVaultAggregated;
pub use non_fungible_resources::NonFungibleResourceVaultAggregated;
use serde::{Deserialize, Serialize};

use self::{
    details::Details, explicit_metadata::ExplicitMetadata,
    fungible_resources::FungibleResourcesVaultAggregated, metadata::Metadata,
    non_fungible_resources::NonFungibleResourcesVaultAggregated,
};

use super::ledger_state::LedgerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountsDetails {
    pub ledger_state: LedgerState,
    pub items: Vec<Account>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub fungible_resources: FungibleResourcesVaultAggregated,
    pub non_fungible_resources: NonFungibleResourcesVaultAggregated,
    pub metadata: Metadata,
    pub explicit_metadata: ExplicitMetadata,
    pub details: Details,
}
