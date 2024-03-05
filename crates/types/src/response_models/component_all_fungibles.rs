
use crate::{AccountAddress, ResourceAddress};
use super::ledger_state::LedgerState;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AllFungiblesResponse {
    pub ledger_state: LedgerState,
    pub total_count: u32,
    pub items: Vec<Fungible>,
    pub address: AccountAddress,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Fungible {
    pub amount: String,
    pub last_updated_at_state_version: u64,
    pub aggregation_level: String,
    pub resource_address: ResourceAddress,
}




