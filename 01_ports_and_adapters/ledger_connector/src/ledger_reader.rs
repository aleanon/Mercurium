//! Read-side ledger port and its Radix gateway adapter.
//!
//! Completes the hexagonal ledger layer alongside [`crate::transaction_gateway`]: the wallet
//! depends on [`LedgerReader`] to refresh account balances/assets, and [`RadixGateway`] (the same
//! adapter that submits transactions) implements it by delegating to the existing gateway
//! update logic in `handles`.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use data_stores::AppDataDb;

use crate::transaction_gateway::RadixGateway;

type AccountsUpdate = types::collections::AccountsUpdate;
type Account = types::Account;
type Resource = types::Resource;
type ResourceAddress = types::address::ResourceAddress;
type AccountAddress = types::address::AccountAddress;
type Transaction = types::Transaction;

#[derive(Debug)]
pub enum LedgerReaderError {
    Update(String),
}

impl std::fmt::Display for LedgerReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Update(msg) => write!(f, "ledger update failed: {msg}"),
        }
    }
}

impl std::error::Error for LedgerReaderError {}

/// Port for refreshing on-ledger state (balances, fungibles, non-fungibles) for the wallet's
/// accounts.
#[async_trait]
pub trait LedgerReader {
    /// Fetches an up-to-date snapshot for all of the wallet's stored accounts. The caller supplies
    /// the open database handle (held by the `Unlocked` wallet state).
    async fn update_all_accounts(
        &self,
        db: &AppDataDb,
    ) -> Result<AccountsUpdate, LedgerReaderError>;

    /// Fetches snapshots for the given accounts (e.g. freshly-derived accounts during setup,
    /// before they are stored), seeded with any already-known resources.
    async fn update_accounts(
        &self,
        accounts: Vec<Account>,
        known_resources: HashMap<ResourceAddress, Resource>,
    ) -> Result<AccountsUpdate, LedgerReaderError>;

    /// Fetches recent committed transactions for an account (newest gateway page).
    async fn transaction_history(
        &self,
        account: AccountAddress,
        from_state_version: Option<i64>,
    ) -> Result<Vec<Transaction>, LedgerReaderError>;
}

#[async_trait]
impl LedgerReader for RadixGateway {
    async fn update_all_accounts(
        &self,
        db: &AppDataDb,
    ) -> Result<AccountsUpdate, LedgerReaderError> {
        crate::radix_dlt::updates::update_all_accounts(self.network, db)
            .await
            .map_err(|e| LedgerReaderError::Update(e.to_string()))
    }

    async fn update_accounts(
        &self,
        accounts: Vec<Account>,
        known_resources: HashMap<ResourceAddress, Resource>,
    ) -> Result<AccountsUpdate, LedgerReaderError> {
        // `update_accounts` is infallible (it logs and skips failures per account).
        Ok(crate::radix_dlt::updates::update_accounts(
            self.network,
            Arc::new(known_resources),
            accounts,
        )
        .await)
    }

    async fn transaction_history(
        &self,
        account: AccountAddress,
        from_state_version: Option<i64>,
    ) -> Result<Vec<Transaction>, LedgerReaderError> {
        crate::radix_dlt::updates::fetch_transactions(self.network, &account, from_state_version)
            .await
            .map_err(|e| LedgerReaderError::Update(e.to_string()))
    }
}
