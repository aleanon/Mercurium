//! Build, sign, notarize and compile a Radix **V1** transaction from a manifest.
//!
//! For a simple wallet transfer the sending account's key is also the notary, and
//! `notary_is_signatory = true`, so the single notary signature satisfies the account's
//! authorization — no separate intent signature is required. The output is the compiled
//! notarized-transaction hex (for gateway submission) plus the bech32m transaction id (the
//! transaction-intent hash, used for status polling).

use deps::*;

use radix_common::prelude::PublicKey;
use radix_transactions::model::TransactionPayload;
use radix_transactions::prelude::{
    HasTransactionIntentHash, PreparationSettings, TransactionBuilder, TransactionHashBech32Encoder,
    TransactionHeaderV1, TransactionManifestV1,
};
use radix_common::types::Epoch;

use types::{Network, crypto::Ed25519KeyPair};

/// Number of epochs the transaction remains valid for after the current epoch.
/// One epoch is roughly 5 minutes, so 10 epochs ≈ 50 minutes.
const EPOCH_VALIDITY_WINDOW: u64 = 10;

#[derive(Debug, thiserror::Error)]
pub enum TransactionBuildError {
    #[error("Failed to encode notarized transaction: {0}")]
    Encode(String),
    #[error("Failed to prepare transaction: {0}")]
    Prepare(String),
    #[error("Failed to encode transaction id: {0}")]
    EncodeId(String),
}

/// A compiled, ready-to-submit transaction.
#[derive(Debug, Clone)]
pub struct CompiledTransaction {
    /// Hex of the SBOR-encoded notarized transaction, for the gateway `/transaction/submit` body.
    pub notarized_hex: String,
    /// The bech32m transaction id (`txid_...`), i.e. the transaction-intent hash, for status polling.
    pub intent_hash_id: String,
}

/// Builds and notarizes a single-signer transfer transaction.
///
/// `signer` is both the account authorizing the withdrawals and the notary.
pub fn build_notarized_transaction(
    manifest: TransactionManifestV1,
    network: Network,
    signer: &Ed25519KeyPair,
    current_epoch: u64,
    nonce: u32,
    tip_percentage: u16,
) -> Result<CompiledTransaction, TransactionBuildError> {
    let network_definition = network.definition();
    let notary_key = signer.radix_private_key();

    let header = TransactionHeaderV1 {
        network_id: network_definition.id,
        start_epoch_inclusive: Epoch::of(current_epoch),
        end_epoch_exclusive: Epoch::of(current_epoch + EPOCH_VALIDITY_WINDOW),
        nonce,
        notary_public_key: PublicKey::Ed25519(notary_key.public_key()),
        notary_is_signatory: true,
        tip_percentage,
    };

    let notarized = TransactionBuilder::new()
        .header(header)
        .manifest(manifest)
        .notarize(&notary_key)
        .build();

    // Prepare first (borrows) to obtain the intent hash, then consume into the raw payload.
    let prepared = notarized
        .prepare(PreparationSettings::latest_ref())
        .map_err(|e| TransactionBuildError::Prepare(format!("{e:?}")))?;
    let intent_hash = prepared.transaction_intent_hash();

    let raw = notarized
        .to_raw()
        .map_err(|e| TransactionBuildError::Encode(format!("{e:?}")))?;
    let notarized_hex = raw.to_hex();

    let intent_hash_id = TransactionHashBech32Encoder::new(&network_definition)
        .encode(&intent_hash)
        .map_err(|e| TransactionBuildError::EncodeId(format!("{e:?}")))?;

    Ok(CompiledTransaction {
        notarized_hex,
        intent_hash_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::manifest::{
        FungibleTransfer, RecipientTransfer, TransferRequest, build_transfer_manifest,
    };
    use radix_common::math::Decimal;
    use std::str::FromStr;
    use types::{
        address::{AccountAddress, ResourceAddress},
        crypto::{Bip32Entity, Bip32KeyKind},
    };

    const STOKENET_XRD: &str =
        "resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc";

    fn test_keypair() -> Ed25519KeyPair {
        use deps::bip39::{Language, Mnemonic};
        let mnemonic = Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap();
        Ed25519KeyPair::new(
            &mnemonic,
            None,
            0,
            Network::Stokenet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        )
        .0
    }

    #[test]
    fn builds_and_notarizes_a_transfer() {
        let signer = test_keypair();
        // The sender is the keypair's own virtual account.
        let from = AccountAddress::from_str(&signer.bech32_address()).unwrap();
        let to = AccountAddress::from_str(
            "account_tdx_2_12866llg04px7q2wee02yxcxtdwgtpzdc8n75fermd070u64t98jtnj",
        )
        .unwrap();

        let request = TransferRequest {
            from,
            transfers: vec![RecipientTransfer {
                to,
                fungibles: vec![FungibleTransfer {
                    resource: ResourceAddress::from_str(STOKENET_XRD).unwrap(),
                    amount: Decimal::from(25),
                }],
                non_fungibles: vec![],
            }],
        };

        let manifest = build_transfer_manifest(&request, Network::Stokenet).unwrap();

        let compiled =
            build_notarized_transaction(manifest, Network::Stokenet, &signer, 1000, 42, 0)
                .expect("transaction builds");

        // Notarized payload should be non-empty valid hex, and the txid should be a stokenet txid.
        assert!(!compiled.notarized_hex.is_empty());
        assert!(compiled.notarized_hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(
            compiled.intent_hash_id.starts_with("txid_tdx_2_"),
            "got: {}",
            compiled.intent_hash_id
        );
    }
}
