//! Radix Connect — connecting Mercurium to dApps.
//!
//! MVP transport is the **HTTPS Connect Relay** (plan §4). This module implements the parts
//! that can be built and tested independently of a live relay:
//!
//! * the wallet-side **domain model** of a dApp interaction (a CAP-21 wallet interaction), and
//! * the **dispatch logic** that turns each request into a response by reusing existing
//!   backends — authentication via ROLA, transactions via the manifest/notarize/submit pipeline.
//!
//! Deliberately **not** implemented here (require the official Radix Connect spec + a running
//! relay to verify, and must not be guessed):
//! * the encrypted relay transport envelope and the pairing/handshake, behind [`RelayTransport`];
//! * byte/JSON-exact CAP-21 (de)serialization for wire interop.
//!
//! The types below are a faithful internal representation, not a wire format.

pub mod cap21;
pub mod relay;

use deps::*;

use bip39::Mnemonic;
use types::{Account, Network, Persona};

use crate::transaction::{
    build::{CompiledTransaction, TransactionBuildError, build_notarized_transaction},
    manifest::compile_manifest_string,
    rola::{RolaChallenge, SignedRolaChallenge, sign_rola_challenge},
};
use crate::wallet::{signing_keypair_for_account, signing_keypair_for_persona};

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Failed to compile dApp manifest: {0}")]
    ManifestCompile(String),
    #[error(transparent)]
    TransactionBuild(#[from] TransactionBuildError),
    #[error("No persona was selected to authenticate with")]
    NoPersonaSelected,
    #[error("Transport error: {0}")]
    Transport(String),
}

/// Metadata identifying the connected dApp and the network of the interaction.
#[derive(Debug, Clone)]
pub struct DappMetadata {
    pub dapp_definition_address: String,
    pub origin: String,
    pub network: Network,
}

/// A request from a connected dApp (a subset of CAP-21 wallet interactions for the MVP).
#[derive(Debug, Clone)]
pub enum WalletRequest {
    /// Authenticate (login) by proving control of a persona against a dApp challenge (ROLA).
    AuthLogin { challenge: RolaChallenge },
    /// A transaction the dApp wants the wallet to review, sign and submit. The manifest is
    /// supplied as RTM text.
    SendTransaction {
        manifest_rtm: String,
        message: Option<String>,
    },
}

/// A full incoming interaction: an id, the dApp metadata, and the requested item.
#[derive(Debug, Clone)]
pub struct WalletInteraction {
    pub interaction_id: String,
    pub metadata: DappMetadata,
    pub request: WalletRequest,
}

/// The wallet's response to an interaction, to be sent back over the relay.
#[derive(Debug, Clone)]
pub enum WalletResponse {
    Auth {
        persona_identity_address: String,
        proof: SignedRolaChallenge,
    },
    Transaction {
        intent_hash_id: String,
    },
    Rejected {
        reason: String,
    },
}

/// Produces a ROLA proof authenticating `persona` against the dApp that issued `challenge`.
///
/// Pure (no network): the dApp's backend later verifies the returned signature.
pub fn authenticate_persona(
    persona: &Persona,
    challenge: &RolaChallenge,
    metadata: &DappMetadata,
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
) -> SignedRolaChallenge {
    let keypair = signing_keypair_for_persona(mnemonic, seed_password, persona);
    sign_rola_challenge(
        &keypair,
        challenge,
        &metadata.dapp_definition_address,
        &metadata.origin,
    )
}

/// Compiles, signs and notarizes a dApp-supplied manifest, ready for submission.
///
/// Pure (no network): the caller submits the returned [`CompiledTransaction`] via
/// [`crate::transaction::service`]. `signer` is the account chosen to notarize/pay.
pub fn prepare_dapp_transaction(
    manifest_rtm: &str,
    metadata: &DappMetadata,
    signer_account: &Account,
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    current_epoch: u64,
    nonce: u32,
    tip_percentage: u16,
) -> Result<CompiledTransaction, ConnectError> {
    let manifest = compile_manifest_string(manifest_rtm, metadata.network)
        .map_err(|e| ConnectError::ManifestCompile(format!("{e:?}")))?;

    let signer = signing_keypair_for_account(mnemonic, seed_password, signer_account);

    let compiled = build_notarized_transaction(
        manifest,
        metadata.network,
        &signer,
        current_epoch,
        nonce,
        tip_percentage,
    )?;

    Ok(compiled)
}

/// Abstraction over the Radix Connect transport (the HTTPS relay for the MVP).
///
/// Implementations own pairing, the encrypted message envelope, and long-polling the relay.
/// This trait is intentionally minimal so the dispatch logic above can be unit-tested without a
/// live relay. A concrete `ConnectRelay` adapter is future work (plan Phase 6) and must conform
/// to the official Radix Connect protocol.
pub trait RelayTransport {
    /// Awaits the next decrypted interaction from a paired dApp.
    fn next_interaction(
        &mut self,
    ) -> impl std::future::Future<Output = Result<WalletInteraction, ConnectError>> + Send;

    /// Encrypts and sends a response for the given interaction id.
    fn send_response(
        &mut self,
        interaction_id: &str,
        response: &WalletResponse,
    ) -> impl std::future::Future<Output = Result<(), ConnectError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use radix_common::crypto::verify_ed25519;
    use std::str::FromStr;
    use types::address::AccountAddress;
    use types::crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair};

    fn mnemonic() -> Mnemonic {
        use bip39::Language;
        Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap()
    }

    fn metadata() -> DappMetadata {
        DappMetadata {
            dapp_definition_address:
                "account_tdx_2_12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66".to_string(),
            origin: "https://dapp.example".to_string(),
            network: Network::Stokenet,
        }
    }

    #[test]
    fn authenticates_a_persona_with_rola() {
        let persona = crate::wallet::create_persona_from_mnemonic(
            &mnemonic(),
            None,
            0,
            0,
            "Persona".to_string(),
            Network::Stokenet,
        );
        let metadata = metadata();
        let challenge: RolaChallenge = [9u8; 32];

        let proof = authenticate_persona(&persona, &challenge, &metadata, &mnemonic(), None);

        // Verifies against the persona's public key and the dApp-bound payload.
        let payload = crate::transaction::rola::rola_payload(
            &challenge,
            &metadata.dapp_definition_address,
            &metadata.origin,
        );
        let message_hash = radix_common::crypto::hash(payload);
        assert!(verify_ed25519(
            &message_hash,
            &proof.public_key,
            &proof.signature
        ));
        assert_eq!(proof.public_key, persona.public_key);
    }

    #[test]
    fn prepares_a_dapp_transaction() {
        // A signing account derived from the same mnemonic.
        let signer_keypair: Ed25519KeyPair = Ed25519KeyPair::new(
            &mnemonic(),
            None,
            0,
            Network::Stokenet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        )
        .0;
        let from = signer_keypair.bech32_address();
        let account = types::Account::new(
            0,
            "Acct".to_string(),
            Network::Stokenet,
            [44 + 0x8000_0000, 1022 + 0x8000_0000, 2, 525, 1460, 0],
            AccountAddress::from_str(&from).expect("valid account address"),
            signer_keypair.radixdlt_public_key(),
        );

        // A simple dApp manifest withdrawing XRD and depositing back to the same account.
        let manifest_rtm = format!(
            r#"CALL_METHOD Address("{from}") "withdraw" Address("resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc") Decimal("1");
TAKE_ALL_FROM_WORKTOP Address("resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc") Bucket("b");
CALL_METHOD Address("{from}") "try_deposit_or_abort" Bucket("b") Enum<0u8>();"#
        );

        let compiled = prepare_dapp_transaction(
            &manifest_rtm,
            &metadata(),
            &account,
            &mnemonic(),
            None,
            1000,
            7,
            0,
        )
        .expect("dApp transaction prepares");

        assert!(compiled.intent_hash_id.starts_with("txid_tdx_2_"));
        assert!(!compiled.notarized_hex.is_empty());
    }
}
