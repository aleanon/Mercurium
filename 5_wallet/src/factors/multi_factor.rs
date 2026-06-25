//! Multi-factor signing — collecting the signatures that satisfy a Security Shield role.
//!
//! An on-ledger Access Controller (Smart Account) guards each role (primary / recovery /
//! confirmation) with a [`RoleOfFactors`]: a *threshold* over some factors, plus *override*
//! factors any one of which satisfies the role alone. To exercise a role the wallet must gather
//! enough signatures over the same payload from the right factors.
//!
//! The signature *production* per factor goes through [`SigningFactor`](super::SigningFactor)
//! (device today, Ledger when a transport is connected). This module is the *aggregation* policy:
//! given the signatures collected so far, pick the subset that satisfies a role, or report exactly
//! what is still missing. It is pure logic and fully unit-tested; the on-ledger semantics of the
//! resulting signature set must still be validated against a live network.

use deps::*;

use std::collections::HashMap;

use radix_common::crypto::Ed25519Signature;
use types::RoleOfFactors;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MfaError {
    #[error("role is not satisfiable as configured (threshold exceeds factors, no override)")]
    RoleUnsatisfiable,
    #[error("not enough signatures: need {needed}, have {got}")]
    InsufficientSignatures { needed: usize, got: usize },
}

/// Selects the signatures that satisfy `role` from those `collected` so far (keyed by factor id).
///
/// Resolution order mirrors the role's own semantics:
/// 1. **Override** — if any override factor has produced a signature, that one signature satisfies
///    the role on its own.
/// 2. **Threshold** — otherwise, if at least `threshold` of the threshold factors have signed,
///    return exactly `threshold` of those signatures.
///
/// Returns [`MfaError::RoleUnsatisfiable`] if the role can never be met as configured, or
/// [`MfaError::InsufficientSignatures`] if it could be met but not enough signatures are in yet.
pub fn signatures_satisfying_role(
    role: &RoleOfFactors,
    collected: &HashMap<String, Ed25519Signature>,
) -> Result<Vec<Ed25519Signature>, MfaError> {
    if !role.is_satisfiable() {
        return Err(MfaError::RoleUnsatisfiable);
    }

    // An override factor satisfies the role by itself.
    for factor in &role.override_factors {
        if let Some(signature) = collected.get(factor) {
            return Ok(vec![*signature]);
        }
    }

    // Otherwise gather signatures from the threshold factors, in declared order for determinism.
    let present: Vec<Ed25519Signature> = role
        .threshold_factors
        .iter()
        .filter_map(|factor| collected.get(factor).copied())
        .collect();

    let needed = role.threshold as usize;
    if present.len() >= needed && needed >= 1 {
        Ok(present.into_iter().take(needed).collect())
    } else {
        Err(MfaError::InsufficientSignatures { needed, got: present.len() })
    }
}

/// Whether the signatures collected so far already satisfy the role (convenience over
/// [`signatures_satisfying_role`]).
pub fn role_is_met(role: &RoleOfFactors, collected: &HashMap<String, Ed25519Signature>) -> bool {
    signatures_satisfying_role(role, collected).is_ok()
}

/// The factor ids still required for `role`, given what has been `collected`. Empty when the role
/// is already met. Reports the override factors (any one suffices) when no threshold path remains.
pub fn outstanding_factors(
    role: &RoleOfFactors,
    collected: &HashMap<String, Ed25519Signature>,
) -> Vec<String> {
    if role_is_met(role, collected) {
        return Vec::new();
    }
    let mut missing: Vec<String> = role
        .threshold_factors
        .iter()
        .filter(|factor| !collected.contains_key(*factor))
        .cloned()
        .collect();
    missing.extend(role.override_factors.iter().cloned());
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(n: u8) -> Ed25519Signature {
        Ed25519Signature([n; 64])
    }

    fn collected(pairs: &[(&str, u8)]) -> HashMap<String, Ed25519Signature> {
        pairs.iter().map(|(id, n)| (id.to_string(), sig(*n))).collect()
    }

    #[test]
    fn single_factor_role_met_by_its_signature() {
        let role = RoleOfFactors::single("device");
        let signatures = signatures_satisfying_role(&role, &collected(&[("device", 1)])).unwrap();
        assert_eq!(signatures, vec![sig(1)]);
    }

    #[test]
    fn override_factor_satisfies_alone() {
        let role = RoleOfFactors {
            threshold: 2,
            threshold_factors: vec!["a".into(), "b".into()],
            override_factors: vec!["recovery".into()],
        };
        // Only the override signed, yet the role is met with that single signature.
        let signatures =
            signatures_satisfying_role(&role, &collected(&[("recovery", 9)])).unwrap();
        assert_eq!(signatures, vec![sig(9)]);
    }

    #[test]
    fn threshold_met_returns_exactly_threshold_signatures() {
        let role = RoleOfFactors {
            threshold: 2,
            threshold_factors: vec!["a".into(), "b".into(), "c".into()],
            override_factors: vec![],
        };
        let signatures =
            signatures_satisfying_role(&role, &collected(&[("a", 1), ("b", 2), ("c", 3)])).unwrap();
        // First two in declared order.
        assert_eq!(signatures, vec![sig(1), sig(2)]);
    }

    #[test]
    fn threshold_not_met_reports_shortfall() {
        let role = RoleOfFactors {
            threshold: 2,
            threshold_factors: vec!["a".into(), "b".into()],
            override_factors: vec![],
        };
        let err = signatures_satisfying_role(&role, &collected(&[("a", 1)])).unwrap_err();
        assert_eq!(err, MfaError::InsufficientSignatures { needed: 2, got: 1 });
    }

    #[test]
    fn unsatisfiable_role_is_rejected() {
        let role = RoleOfFactors {
            threshold: 3,
            threshold_factors: vec!["a".into()],
            override_factors: vec![],
        };
        assert_eq!(
            signatures_satisfying_role(&role, &collected(&[("a", 1)])),
            Err(MfaError::RoleUnsatisfiable),
        );
    }

    #[test]
    fn outstanding_factors_lists_what_is_missing_then_empties_when_met() {
        let role = RoleOfFactors {
            threshold: 2,
            threshold_factors: vec!["a".into(), "b".into()],
            override_factors: vec!["recovery".into()],
        };
        let mut have = collected(&[("a", 1)]);
        let missing = outstanding_factors(&role, &have);
        assert!(missing.contains(&"b".to_string()));
        assert!(missing.contains(&"recovery".to_string()));

        have.insert("b".to_string(), sig(2));
        assert!(role_is_met(&role, &have));
        assert!(outstanding_factors(&role, &have).is_empty());
    }
}
