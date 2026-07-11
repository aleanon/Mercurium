//! Transaction-submission port and its Radix gateway adapter.
//!
//! This is the write-side of the ledger connector (the read-side lives in `ledger_reader.rs`
//! and `radix_dlt`). It is the hexagonal seam for committing transactions: the wallet depends on
//! the [`TransactionGateway`] trait, and [`RadixGateway`] is the concrete adapter backed by the
//! Radix Gateway API.

use async_trait::async_trait;

type Network = types::Network;

#[derive(Debug)]
pub enum TransactionGatewayError {
    Gateway(String),
}

impl std::fmt::Display for TransactionGatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gateway(msg) => write!(f, "gateway error: {msg}"),
        }
    }
}

impl std::error::Error for TransactionGatewayError {}

/// The overall status of a submitted transaction, as reported by the gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmittedStatus {
    Pending,
    CommittedSuccess,
    CommittedFailure,
    Rejected,
    Unknown,
}

/// Port for fetching construction metadata, submitting transactions, and polling their status.
#[async_trait]
pub trait TransactionGateway {
    /// The current epoch, needed to build a transaction header's validity window.
    async fn current_epoch(&self) -> Result<u64, TransactionGatewayError>;

    /// Submits a notarized transaction (SBOR-encoded, hex). Returns `true` if it was a
    /// duplicate of an already-pending transaction.
    async fn submit(&self, notarized_hex: &str) -> Result<bool, TransactionGatewayError>;

    /// Returns the status of a transaction identified by its bech32m `txid_...` intent hash.
    async fn status(
        &self,
        intent_hash_id: &str,
    ) -> Result<SubmittedStatus, TransactionGatewayError>;
}

/// Concrete adapter backed by the Radix Gateway API (via the `handles` gateway requests).
pub struct RadixGateway {
    pub network: Network,
}

impl RadixGateway {
    pub fn new(network: Network) -> Self {
        Self { network }
    }
}

#[async_trait]
impl TransactionGateway for RadixGateway {
    async fn current_epoch(&self) -> Result<u64, TransactionGatewayError> {
        crate::radix_dlt::gateway_requests::get_current_epoch(self.network)
            .await
            .map_err(|e| TransactionGatewayError::Gateway(e.to_string()))
    }

    async fn submit(&self, notarized_hex: &str) -> Result<bool, TransactionGatewayError> {
        let response =
            crate::radix_dlt::gateway_requests::submit_transaction(self.network, notarized_hex)
                .await
                .map_err(|e| TransactionGatewayError::Gateway(e.to_string()))?;
        Ok(response.duplicate)
    }

    async fn status(
        &self,
        intent_hash_id: &str,
    ) -> Result<SubmittedStatus, TransactionGatewayError> {
        use deps::radix_gateway_sdk::generated::model::TransactionStatus;

        let response = crate::radix_dlt::gateway_requests::get_transaction_status(
            self.network,
            intent_hash_id,
        )
        .await
        .map_err(|e| TransactionGatewayError::Gateway(e.to_string()))?;

        Ok(match response.status {
            TransactionStatus::CommittedSuccess => SubmittedStatus::CommittedSuccess,
            TransactionStatus::CommittedFailure => SubmittedStatus::CommittedFailure,
            TransactionStatus::Pending => SubmittedStatus::Pending,
            TransactionStatus::Rejected => SubmittedStatus::Rejected,
            TransactionStatus::Unknown => SubmittedStatus::Unknown,
        })
    }
}
