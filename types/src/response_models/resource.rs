use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceDetails {
    pub divisibility: Option<u8>,
    pub total_supply: String,
}
