use serde::{Serialize, Deserialize};

use super::ledger_state::LedgerState;


#[derive(Debug, Serialize, Deserialize)]
pub struct AllNFTsResponse {
    pub ledger_state: LedgerState,
    pub total_count: usize,
    pub items: Vec<NonFungible>,
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungible {
    pub amount: u64,
    pub last_updated_at_state_version: u64,
    pub aggregation_level: String,
    pub resource_address: String,
}