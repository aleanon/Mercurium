//! Read-side ledger port and its Radix gateway adapter.
//!
//! Completes the hexagonal ledger layer alongside [`crate::transaction_gateway`]: the wallet
//! depends on [`LedgerReader`] to refresh account balances/assets, and [`RadixGateway`] (the same
//! adapter that submits transactions) implements it by delegating to the existing gateway
//! update logic in `handles`.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::transaction_gateway::RadixGateway;

type AccountsUpdate = types::collections::AccountsUpdate;
type Account = types::Account;
type Resource = types::Resource;
type ResourceAddress = types::address::ResourceAddress;

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
    /// Fetches an up-to-date snapshot for all of the wallet's stored accounts.
    async fn update_all_accounts(&self) -> Result<AccountsUpdate, LedgerReaderError>;

    /// Fetches snapshots for the given accounts (e.g. freshly-derived accounts during setup,
    /// before they are stored), seeded with any already-known resources.
    async fn update_accounts(
        &self,
        accounts: Vec<Account>,
        known_resources: HashMap<ResourceAddress, Resource>,
    ) -> Result<AccountsUpdate, LedgerReaderError>;
}

#[async_trait]
impl LedgerReader for RadixGateway {
    async fn update_all_accounts(&self) -> Result<AccountsUpdate, LedgerReaderError> {
        handles::radix_dlt::updates::update_all_accounts(self.network)
            .await
            .map_err(|e| LedgerReaderError::Update(e.to_string()))
    }

    async fn update_accounts(
        &self,
        accounts: Vec<Account>,
        known_resources: HashMap<ResourceAddress, Resource>,
    ) -> Result<AccountsUpdate, LedgerReaderError> {
        // `handles::update_accounts` is infallible (it logs and skips failures per account).
        Ok(handles::radix_dlt::updates::update_accounts(
            self.network,
            Arc::new(known_resources),
            accounts,
        )
        .await)
    }
}
