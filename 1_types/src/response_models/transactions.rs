use deps_two::*;

use serde::{Deserialize, Serialize};

use super::LedgerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub ledger_state: LedgerState,
    pub items: Vec<TransactionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction_status: String,
    pub state_version: u64,
    pub confirmed_at: String,
    pub message: Option<Message>,
    pub balance_changes: Option<BalanceChanges>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "type")]
    pub message_type: String,
    pub content: Content,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceChanges {
    pub fungible_fee_balance_changes: Vec<FungibleFeeBalanceChange>,
    pub fungible_balance_changes: Vec<FungibleBalanceChange>,
    pub non_fungible_balance_changes: Vec<NonFungibleBalanceChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleFeeBalanceChange {
    pub entity_address: String,
    pub resource_address: String,
    pub balance_change: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleBalanceChange {
    pub entity_address: String,
    pub resource_address: String,
    pub balance_change: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleBalanceChange {
    pub entity_address: String,
    pub resource_address: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}
