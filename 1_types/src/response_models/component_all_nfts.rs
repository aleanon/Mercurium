use serde::{Deserialize, Serialize};

use super::ledger_state::LedgerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AllNFTsResponse {
    pub ledger_state: LedgerState,
    pub total_count: usize,
    pub items: Vec<NonFungibleResponse>,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleResponse {
    pub amount: u64,
    pub last_updated_at_state_version: u64,
    pub aggregation_level: String,
    pub resource_address: String,
}
