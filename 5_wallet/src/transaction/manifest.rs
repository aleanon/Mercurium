//! Construction of Radix transaction manifests from a wallet-level transfer request.
//!
//! This module is intentionally pure: it turns a [`TransferRequest`] (built from the UI's
//! recipient/asset model) into a `TransactionManifestV1` and, optionally, a human-readable
//! manifest string for a confirmation screen. It performs no network or signing I/O — that
//! lives in the transaction service (Phase 3).

use deps::*;

use radix_common::math::Decimal;
use radix_common::prelude::NonFungibleLocalId;
use radix_common::types::{ComponentAddress, ResourceAddress as EngineResourceAddress};
use radix_transactions::builder::ManifestBuilder;
use radix_transactions::model::TransactionManifestV1;
use scrypto::address::AddressBech32Decoder;

use types::{
    Network,
    address::{AccountAddress, Address, ResourceAddress},
};

#[derive(Debug, thiserror::Error)]
pub enum ManifestBuildError {
    #[error("Invalid account address in transfer: {0}")]
    InvalidAccountAddress(String),
    #[error("Invalid resource address in transfer: {0}")]
    InvalidResourceAddress(String),
    #[error("Transfer request contained no assets to send")]
    Empty,
}

/// A wallet-level request to transfer assets from one account to one or more recipients.
#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub from: AccountAddress,
    pub transfers: Vec<RecipientTransfer>,
}

/// The assets destined for a single recipient account.
#[derive(Debug, Clone)]
pub struct RecipientTransfer {
    pub to: AccountAddress,
    pub fungibles: Vec<FungibleTransfer>,
    pub non_fungibles: Vec<NonFungibleTransfer>,
}

#[derive(Debug, Clone)]
pub struct FungibleTransfer {
    pub resource: ResourceAddress,
    pub amount: Decimal,
}

#[derive(Debug, Clone)]
pub struct NonFungibleTransfer {
    pub resource: ResourceAddress,
    pub ids: Vec<NonFungibleLocalId>,
}

impl TransferRequest {
    fn is_empty(&self) -> bool {
        self.transfers
            .iter()
            .all(|t| t.fungibles.is_empty() && t.non_fungibles.is_empty())
    }
}

/// Builds a `TransactionManifestV1` that withdraws each requested asset from the sending
/// account and deposits it into the recipient account, aborting if any recipient refuses the
/// deposit (`try_deposit_or_abort`).
pub fn build_transfer_manifest(
    request: &TransferRequest,
    network: Network,
) -> Result<TransactionManifestV1, ManifestBuildError> {
    if request.is_empty() {
        return Err(ManifestBuildError::Empty);
    }

    let decoder = AddressBech32Decoder::new(&network.definition());

    let from = decode_account(&decoder, &request.from)?;

    let mut builder = ManifestBuilder::new();
    let mut bucket_index = 0usize;

    for transfer in &request.transfers {
        let to = decode_account(&decoder, &transfer.to)?;

        for fungible in &transfer.fungibles {
            let resource = decode_resource(&decoder, &fungible.resource)?;
            let bucket = format!("bucket_{bucket_index}");
            bucket_index += 1;

            builder = builder
                .withdraw_from_account(from, resource, fungible.amount)
                .take_from_worktop(resource, fungible.amount, bucket.as_str())
                .try_deposit_or_abort(to, None, bucket.as_str());
        }

        for non_fungible in &transfer.non_fungibles {
            if non_fungible.ids.is_empty() {
                continue;
            }
            let resource = decode_resource(&decoder, &non_fungible.resource)?;
            let bucket = format!("bucket_{bucket_index}");
            bucket_index += 1;

            builder = builder
                .withdraw_non_fungibles_from_account(
                    from,
                    resource,
                    non_fungible.ids.iter().cloned(),
                )
                .take_non_fungibles_from_worktop(
                    resource,
                    non_fungible.ids.iter().cloned(),
                    bucket.as_str(),
                )
                .try_deposit_or_abort(to, None, bucket.as_str());
        }
    }

    Ok(builder.build())
}

/// Decompiles a manifest into the human-readable RTM (Radix Transaction Manifest) text used
/// on a confirmation/review screen.
pub fn manifest_to_string(
    manifest: &TransactionManifestV1,
    network: Network,
) -> Result<String, radix_transactions::manifest::DecompileError> {
    radix_transactions::manifest::decompile(manifest, &network.definition())
}

/// Compiles a dApp-supplied RTM string into a `TransactionManifestV1`.
///
/// This is the entry point for Radix Connect **transaction requests**, where a connected dApp
/// sends a manifest as text for the wallet to review, sign and submit. Blobs are not supported
/// for the MVP (an empty blob provider is used).
pub fn compile_manifest_string(
    manifest_string: &str,
    network: Network,
) -> Result<TransactionManifestV1, radix_transactions::manifest::CompileError> {
    radix_transactions::manifest::compile_manifest_v1(
        manifest_string,
        &network.definition(),
        radix_transactions::manifest::BlobProvider::new(),
    )
}

fn decode_account(
    decoder: &AddressBech32Decoder,
    address: &AccountAddress,
) -> Result<ComponentAddress, ManifestBuildError> {
    ComponentAddress::try_from_bech32(decoder, address.as_str())
        .ok_or_else(|| ManifestBuildError::InvalidAccountAddress(address.as_str().to_string()))
}

fn decode_resource(
    decoder: &AddressBech32Decoder,
    address: &ResourceAddress,
) -> Result<EngineResourceAddress, ManifestBuildError> {
    EngineResourceAddress::try_from_bech32(decoder, address.as_str())
        .ok_or_else(|| ManifestBuildError::InvalidResourceAddress(address.as_str().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // XRD on Stokenet.
    const STOKENET_XRD: &str =
        "resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc";
    const STOKENET_ACCOUNT_A: &str =
        "account_tdx_2_12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66";
    const STOKENET_ACCOUNT_B: &str =
        "account_tdx_2_12866llg04px7q2wee02yxcxtdwgtpzdc8n75fermd070u64t98jtnj";

    fn account(addr: &str) -> AccountAddress {
        AccountAddress::from_str(addr).expect("valid account address")
    }

    #[test]
    fn builds_a_fungible_transfer_manifest() {
        let request = TransferRequest {
            from: account(STOKENET_ACCOUNT_A),
            transfers: vec![RecipientTransfer {
                to: account(STOKENET_ACCOUNT_B),
                fungibles: vec![FungibleTransfer {
                    resource: ResourceAddress::from_str(STOKENET_XRD).unwrap(),
                    amount: Decimal::from(100),
                }],
                non_fungibles: vec![],
            }],
        };

        let manifest =
            build_transfer_manifest(&request, Network::Stokenet).expect("manifest builds");

        let rtm = manifest_to_string(&manifest, Network::Stokenet).expect("decompiles");

        // The manifest should withdraw from A and deposit into B.
        assert!(rtm.contains("withdraw"), "manifest:\n{rtm}");
        assert!(rtm.contains("try_deposit_or_abort"), "manifest:\n{rtm}");
        assert!(rtm.contains(STOKENET_ACCOUNT_A), "manifest:\n{rtm}");
        assert!(rtm.contains(STOKENET_ACCOUNT_B), "manifest:\n{rtm}");
    }

    #[test]
    fn dapp_manifest_string_roundtrips() {
        // Build a manifest, render it to RTM, then recompile it as if it came from a dApp.
        let request = TransferRequest {
            from: account(STOKENET_ACCOUNT_A),
            transfers: vec![RecipientTransfer {
                to: account(STOKENET_ACCOUNT_B),
                fungibles: vec![FungibleTransfer {
                    resource: ResourceAddress::from_str(STOKENET_XRD).unwrap(),
                    amount: Decimal::from(5),
                }],
                non_fungibles: vec![],
            }],
        };
        let manifest = build_transfer_manifest(&request, Network::Stokenet).unwrap();
        let rtm = manifest_to_string(&manifest, Network::Stokenet).unwrap();

        let recompiled = compile_manifest_string(&rtm, Network::Stokenet)
            .expect("dApp manifest string compiles");
        // The recompiled manifest should have the same instructions.
        assert_eq!(recompiled.instructions.len(), manifest.instructions.len());
    }

    #[test]
    fn empty_request_is_rejected() {
        let request = TransferRequest {
            from: account(STOKENET_ACCOUNT_A),
            transfers: vec![RecipientTransfer {
                to: account(STOKENET_ACCOUNT_B),
                fungibles: vec![],
                non_fungibles: vec![],
            }],
        };

        assert!(matches!(
            build_transfer_manifest(&request, Network::Stokenet),
            Err(ManifestBuildError::Empty)
        ));
    }
}

#[cfg(test)]
mod non_fungible_tests {
    use super::*;
    use std::str::FromStr;

    const STOKENET_ACCOUNT_A: &str =
        "account_tdx_2_12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66";
    const STOKENET_ACCOUNT_B: &str =
        "account_tdx_2_12866llg04px7q2wee02yxcxtdwgtpzdc8n75fermd070u64t98jtnj";
    // Any well-formed stokenet resource address works for manifest text construction.
    const STOKENET_NFT_RESOURCE: &str =
        "resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc";

    #[test]
    fn builds_a_non_fungible_transfer_manifest() {
        let request = TransferRequest {
            from: AccountAddress::from_str(STOKENET_ACCOUNT_A).unwrap(),
            transfers: vec![RecipientTransfer {
                to: AccountAddress::from_str(STOKENET_ACCOUNT_B).unwrap(),
                fungibles: vec![],
                non_fungibles: vec![NonFungibleTransfer {
                    resource: ResourceAddress::from_str(STOKENET_NFT_RESOURCE).unwrap(),
                    ids: vec![NonFungibleLocalId::integer(1)],
                }],
            }],
        };

        let manifest = build_transfer_manifest(&request, Network::Stokenet).unwrap();
        let rtm = manifest_to_string(&manifest, Network::Stokenet).unwrap();

        assert!(rtm.contains("withdraw_non_fungibles"), "manifest:\n{rtm}");
        assert!(rtm.contains("try_deposit_or_abort"), "manifest:\n{rtm}");
    }

    #[test]
    fn non_fungible_transfer_with_empty_ids_produces_no_instructions() {
        let request = TransferRequest {
            from: AccountAddress::from_str(STOKENET_ACCOUNT_A).unwrap(),
            transfers: vec![RecipientTransfer {
                to: AccountAddress::from_str(STOKENET_ACCOUNT_B).unwrap(),
                fungibles: vec![],
                non_fungibles: vec![NonFungibleTransfer {
                    resource: ResourceAddress::from_str(STOKENET_NFT_RESOURCE).unwrap(),
                    ids: vec![],
                }],
            }],
        };
        // A recipient is present (so the request is not "Empty"), but the empty NFT id list is
        // skipped, leaving a manifest with no instructions.
        let manifest = build_transfer_manifest(&request, Network::Stokenet).unwrap();
        assert_eq!(manifest.instructions.len(), 0);
    }
}
