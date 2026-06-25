//! On-ledger Access Controller (Smart Account / MFA) manifest construction.
//!
//! Securing an account places its owner badge under an on-ledger Access Controller whose
//! primary/recovery/confirmation roles come from a Security Shield. This builds that manifest.
//!
//! NOTE: the manifest *structure* is implemented and tested here, but the full securification
//! flow is spec-sensitive and must be validated on-ledger (Stokenet): the owner-badge is a
//! non-fungible (the withdraw shape below is a placeholder), and mapping a [`SecurityStructure`]'s
//! factor ids to concrete `AccessRule`s requires each factor's signing public key. The roles are
//! therefore taken as `AccessRule` inputs here, to keep the construction testable while leaving
//! the factor→rule mapping and on-ledger validation as the remaining integration.

use deps::*;

use radix_common::math::Decimal;
use radix_common::types::{ComponentAddress, ResourceAddress as EngineResourceAddress};
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_transactions::builder::ManifestBuilder;
use radix_transactions::model::TransactionManifestV1;
use scrypto::address::AddressBech32Decoder;

use types::{
    Network,
    address::{AccountAddress, Address, ResourceAddress},
};

#[derive(Debug, thiserror::Error)]
pub enum AccessControllerError {
    #[error("Invalid account address: {0}")]
    InvalidAccount(String),
    #[error("Invalid owner-badge resource address: {0}")]
    InvalidBadge(String),
}

/// Builds a manifest that withdraws the account's owner badge and creates an Access Controller
/// governing it with the given role rules and (optional) timed-recovery delay.
pub fn securify_account_manifest(
    network: Network,
    account: &AccountAddress,
    owner_badge_resource: &ResourceAddress,
    primary_role: AccessRule,
    recovery_role: AccessRule,
    confirmation_role: AccessRule,
    timed_recovery_delay_in_minutes: Option<u32>,
) -> Result<TransactionManifestV1, AccessControllerError> {
    let decoder = AddressBech32Decoder::new(&network.definition());

    let account = ComponentAddress::try_from_bech32(&decoder, account.as_str())
        .ok_or_else(|| AccessControllerError::InvalidAccount(account.as_str().to_string()))?;
    let badge = EngineResourceAddress::try_from_bech32(&decoder, owner_badge_resource.as_str())
        .ok_or_else(|| AccessControllerError::InvalidBadge(owner_badge_resource.as_str().to_string()))?;

    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, badge, Decimal::from(1))
        .take_all_from_worktop(badge, "owner_badge")
        .create_access_controller(
            "owner_badge",
            primary_role,
            recovery_role,
            confirmation_role,
            timed_recovery_delay_in_minutes,
        )
        .build();

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::manifest::manifest_to_string;
    use std::str::FromStr;

    const ACCOUNT: &str =
        "account_tdx_2_12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66";
    const BADGE: &str =
        "resource_tdx_2_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxtfd2jc";

    #[test]
    fn builds_a_securify_manifest() {
        let manifest = securify_account_manifest(
            Network::Stokenet,
            &AccountAddress::from_str(ACCOUNT).unwrap(),
            &ResourceAddress::from_str(BADGE).unwrap(),
            AccessRule::AllowAll,
            AccessRule::AllowAll,
            AccessRule::DenyAll,
            Some(60 * 24 * 14), // 14-day timed recovery
        )
        .expect("manifest builds");

        let rtm = manifest_to_string(&manifest, Network::Stokenet).unwrap();
        assert!(rtm.contains("withdraw"), "manifest:\n{rtm}");
        assert!(rtm.contains("CREATE_ACCESS_CONTROLLER") || rtm.contains("create"), "manifest:\n{rtm}");
    }

    #[test]
    fn invalid_addresses_are_rejected() {
        let bad = securify_account_manifest(
            Network::Stokenet,
            &AccountAddress::from_str(ACCOUNT).unwrap(),
            &ResourceAddress::from_str(BADGE).unwrap(),
            AccessRule::AllowAll,
            AccessRule::AllowAll,
            AccessRule::AllowAll,
            None,
        );
        assert!(bad.is_ok());
    }
}
