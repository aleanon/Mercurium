//! App lock — a PIN gate for launching the wallet and authorizing sensitive actions.
//!
//! The PIN is never stored; only a PBKDF2-HMAC-SHA256 hash + salt are kept (serializable, so it
//! can live in the profile's security preferences). Verification is constant-time. Biometric
//! unlock is a platform integration that would gate access to this same lock.

use deps::*;

use std::num::NonZeroU32;

use ring::pbkdf2::{self, PBKDF2_HMAC_SHA256};
use serde::{Deserialize, Serialize};
use types::crypto::Salt;

const ITERATIONS: u32 = 120_000;
const HASH_LEN: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum AppLockError {
    #[error("Failed to generate random salt")]
    Random,
}

/// A PIN lock: stores only `salt` + the derived `hash`, never the PIN.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppLock {
    salt: [u8; Salt::LENGTH],
    hash: [u8; HASH_LEN],
}

impl AppLock {
    fn iterations() -> NonZeroU32 {
        NonZeroU32::new(ITERATIONS).expect("non-zero iteration count")
    }

    /// Creates a lock from a PIN (or any passphrase).
    pub fn new(pin: &str) -> Result<Self, AppLockError> {
        let salt = Salt::new().map_err(|_| AppLockError::Random)?;
        let mut hash = [0u8; HASH_LEN];
        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            Self::iterations(),
            salt.as_bytes(),
            pin.as_bytes(),
            &mut hash,
        );
        let salt_bytes = salt.to_inner();
        Ok(Self { salt: salt_bytes, hash })
    }

    /// Constant-time check that `pin` matches the stored hash.
    pub fn verify(&self, pin: &str) -> bool {
        pbkdf2::verify(
            PBKDF2_HMAC_SHA256,
            Self::iterations(),
            &self.salt,
            pin.as_bytes(),
            &self.hash,
        )
        .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_correct_pin_and_rejects_wrong() {
        let lock = AppLock::new("1234").unwrap();
        assert!(lock.verify("1234"));
        assert!(!lock.verify("0000"));
        assert!(!lock.verify("12345"));
        assert!(!lock.verify(""));
    }

    #[test]
    fn two_locks_with_same_pin_have_different_salts() {
        let a = AppLock::new("secret").unwrap();
        let b = AppLock::new("secret").unwrap();
        assert_ne!(a, b); // different random salts
        assert!(a.verify("secret"));
        assert!(b.verify("secret"));
    }

    #[test]
    fn serde_roundtrip_preserves_verification() {
        let lock = AppLock::new("9999").unwrap();
        let json = deps::serde_json::to_string(&lock).unwrap();
        let restored: AppLock = deps::serde_json::from_str(&json).unwrap();
        assert!(restored.verify("9999"));
        assert!(!restored.verify("1111"));
    }
}
