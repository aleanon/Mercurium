use serde::{Deserialize, Serialize};

use super::ledger_state::LedgerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleVaultsResponse {
    ledger_state: LedgerState,
    total_count: u32,
    items: Vec<FungibleVault>,
    address: String,
    resource_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleVault {
    vault_address: String,
    amount: String,
    last_updated_at_state_version: u64,
}