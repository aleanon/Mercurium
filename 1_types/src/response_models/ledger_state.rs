use deps_two::*;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerState {
    network: String,
    state_version: u64,
    proposer_round_timestamp: String,
    epoch: u64,
    round: u32
}