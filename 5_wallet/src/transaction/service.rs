//! Orchestrates the end-to-end "send" path: build a manifest, fetch the current epoch,
//! notarize/compile the transaction, submit it to the gateway, and poll for its outcome.
//!
//! The caller supplies the signing key pair (re-derived from the wallet mnemonic via
//! [`crate::wallet::signing_keypair_for_account`]); this module performs no secret access
//! itself, keeping it independent of the session/credential model (see plan §6).

use deps::*;

use std::time::Duration;

use ledger_connector::{SubmittedStatus, TransactionGateway, radix_transaction_gateway};
use types::{Account, AppError, Network, Notification, crypto::Ed25519KeyPair, crypto::Password};

use crate::wallet::signing_keypair_for_account;

use super::build::{build_notarized_transaction, TransactionBuildError};
use super::manifest::{ManifestBuildError, TransferRequest, build_transfer_manifest};

#[derive(Debug, thiserror::Error)]
pub enum TransactionServiceError {
    #[error(transparent)]
    Manifest(#[from] ManifestBuildError),
    #[error(transparent)]
    Build(#[from] TransactionBuildError),
    #[error("Gateway request failed: {0}")]
    Gateway(String),
}

impl From<radix_gateway_sdk::Error> for TransactionServiceError {
    fn from(e: radix_gateway_sdk::Error) -> Self {
        Self::Gateway(e.to_string())
    }
}

/// Wallet-level transaction status, mapped from the gateway's status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionOutcome {
    Pending,
    CommittedSuccess,
    CommittedFailure,
    Rejected,
    Unknown,
}

impl From<SubmittedStatus> for TransactionOutcome {
    fn from(status: SubmittedStatus) -> Self {
        match status {
            SubmittedStatus::CommittedSuccess => Self::CommittedSuccess,
            SubmittedStatus::CommittedFailure => Self::CommittedFailure,
            SubmittedStatus::Pending => Self::Pending,
            SubmittedStatus::Rejected => Self::Rejected,
            SubmittedStatus::Unknown => Self::Unknown,
        }
    }
}

impl TransactionOutcome {
    /// Whether the transaction has reached a final state and polling can stop.
    pub fn is_settled(&self) -> bool {
        matches!(
            self,
            Self::CommittedSuccess | Self::CommittedFailure | Self::Rejected
        )
    }
}

/// Result of submitting a transaction.
#[derive(Debug, Clone)]
pub struct SubmittedTransaction {
    /// The bech32m transaction id (`txid_...`) used to poll for status.
    pub intent_hash_id: String,
    /// True if the network already had this exact transaction pending.
    pub duplicate: bool,
}

/// Builds, notarizes, and submits a transfer in one call. Does not wait for commitment;
/// use [`poll_until_settled`] (or [`transaction_status`]) with the returned id.
pub async fn submit_transfer(
    request: &TransferRequest,
    network: Network,
    signer: &Ed25519KeyPair,
    tip_percentage: u16,
) -> Result<SubmittedTransaction, TransactionServiceError> {
    let gateway = radix_transaction_gateway(network);

    let manifest = build_transfer_manifest(request, network)?;
    let current_epoch = gateway
        .current_epoch()
        .await
        .map_err(|e| TransactionServiceError::Gateway(e.to_string()))?;
    let nonce: u32 = rand::random();

    let compiled = build_notarized_transaction(
        manifest,
        network,
        signer,
        current_epoch,
        nonce,
        tip_percentage,
    )?;

    let duplicate = gateway
        .submit(&compiled.notarized_hex)
        .await
        .map_err(|e| TransactionServiceError::Gateway(e.to_string()))?;

    Ok(SubmittedTransaction {
        intent_hash_id: compiled.intent_hash_id,
        duplicate,
    })
}

/// UI-facing convenience: decrypt the mnemonic with the login `password`, derive the signing
/// key for `from_account`, and submit `request`. All arguments are owned/`'static`, so this can
/// be driven directly from an async UI task. The mnemonic is dropped (zeroized) when this
/// returns; it is never cached.
pub async fn submit_transfer_with_password(
    request: TransferRequest,
    from_account: Account,
    password: Password,
    tip_percentage: u16,
) -> Result<SubmittedTransaction, AppError> {
    let network = from_account.network;

    let (mnemonic, seed_password) = crate::wallet::get_decrypted_mnemonic(&password)?;
    let signer =
        signing_keypair_for_account(&mnemonic, Some(seed_password.as_str()), &from_account);

    submit_transfer(&request, network, &signer, tip_percentage)
        .await
        .map_err(|err| AppError::NonFatal(Notification::Info(err.to_string())))
}

/// Reads an account's persisted transaction history, owned-data so it can be driven from an
/// async UI task. Populated by gateway sync (`upsert_transactions`).
pub async fn read_account_transactions(
    network: Network,
    account: types::address::AccountAddress,
) -> Result<std::collections::BTreeSet<types::Transaction>, AppError> {
    let db = data_stores::AppDataDb::get(network)
        .ok_or_else(|| AppError::Fatal("Database not found".to_string()))?;
    db.get_transactions_for_account::<std::collections::BTreeSet<types::Transaction>>(account)
        .await
        .map_err(|err| AppError::NonFatal(Notification::Info(err.to_string())))
}

/// Fetches the current outcome of a submitted transaction by its `txid_...` id.
pub async fn transaction_status(
    network: Network,
    intent_hash_id: &str,
) -> Result<TransactionOutcome, TransactionServiceError> {
    let status = radix_transaction_gateway(network)
        .status(intent_hash_id)
        .await
        .map_err(|e| TransactionServiceError::Gateway(e.to_string()))?;
    Ok(status.into())
}

/// Polls the gateway until the transaction settles or the attempt budget is exhausted.
/// Returns the last observed outcome (which may still be `Pending` if it timed out).
pub async fn poll_until_settled(
    network: Network,
    intent_hash_id: &str,
    max_attempts: u32,
    interval: Duration,
) -> Result<TransactionOutcome, TransactionServiceError> {
    let mut last = TransactionOutcome::Unknown;
    for _ in 0..max_attempts {
        last = transaction_status(network, intent_hash_id).await?;
        if last.is_settled() {
            return Ok(last);
        }
        tokio::time::sleep(interval).await;
    }
    Ok(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submitted_status_maps_to_outcome() {
        assert_eq!(
            TransactionOutcome::from(SubmittedStatus::CommittedSuccess),
            TransactionOutcome::CommittedSuccess
        );
        assert_eq!(
            TransactionOutcome::from(SubmittedStatus::CommittedFailure),
            TransactionOutcome::CommittedFailure
        );
        assert_eq!(
            TransactionOutcome::from(SubmittedStatus::Pending),
            TransactionOutcome::Pending
        );
        assert_eq!(
            TransactionOutcome::from(SubmittedStatus::Rejected),
            TransactionOutcome::Rejected
        );
        assert_eq!(
            TransactionOutcome::from(SubmittedStatus::Unknown),
            TransactionOutcome::Unknown
        );
    }

    #[test]
    fn is_settled_only_for_terminal_outcomes() {
        assert!(TransactionOutcome::CommittedSuccess.is_settled());
        assert!(TransactionOutcome::CommittedFailure.is_settled());
        assert!(TransactionOutcome::Rejected.is_settled());
        assert!(!TransactionOutcome::Pending.is_settled());
        assert!(!TransactionOutcome::Unknown.is_settled());
    }
}
