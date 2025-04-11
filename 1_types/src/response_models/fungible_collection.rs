use deps_two::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleCollectionItemGlobal {
    pub amount: String,
    pub last_updated_at_state_version: i64,
    pub aggregation_level: String,
    pub resource_address: String,
}
