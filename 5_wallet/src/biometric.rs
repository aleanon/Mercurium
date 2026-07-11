//! Biometric unlock — an optional, faster gate that stands in front of the [`AppLock`] PIN.
//!
//! Biometrics are inherently platform-specific (Secure Enclave / Touch ID on macOS, Android
//! Keystore + BiometricPrompt, `fprintd`/PAM on Linux), so the actual sensor call can't run in a
//! headless build. This module provides the parts that *are* portable and testable:
//!
//! - [`BiometricAuthenticator`] — the trait a platform backend implements (`availability` +
//!   `authenticate`).
//! - [`UnsupportedBiometric`] — the default backend for platforms/builds with no sensor; always
//!   reports [`BiometricAvailability::Unavailable`].
//! - [`unlock`] — the gating policy: try biometrics when enabled *and* available, and fall back to
//!   the PIN otherwise. The PIN ([`AppLock`]) remains the source of truth, so a failed or absent
//!   sensor never locks the user out.
//!
//! [`AppLock`]: types::AppLock


use types::AppLock;

#[derive(Debug, thiserror::Error)]
pub enum BiometricError {
    #[error("Biometric authentication unavailable: {0}")]
    Unavailable(String),
    #[error("Biometric authentication failed")]
    Failed,
}

/// Whether biometric authentication can be attempted on this device/build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiometricAvailability {
    Available,
    /// Not usable here (no sensor, not enrolled, unsupported platform), with a human-readable why.
    Unavailable(String),
}

/// A platform biometric backend. Implementations call the OS sensor; this crate only depends on
/// the trait so the gating policy stays testable.
pub trait BiometricAuthenticator {
    fn availability(&self) -> BiometricAvailability;

    /// Prompts the user for a biometric match, surfacing `reason` in the OS dialog. `Ok(())` means
    /// the user authenticated; `Err` means it failed or was unavailable.
    fn authenticate(&self, reason: &str) -> Result<(), BiometricError>;
}

/// The default backend for builds/platforms without a biometric sensor (including this headless
/// build). Always unavailable, so callers fall back to the PIN.
pub struct UnsupportedBiometric;

impl BiometricAuthenticator for UnsupportedBiometric {
    fn availability(&self) -> BiometricAvailability {
        BiometricAvailability::Unavailable("no biometric sensor in this build".to_string())
    }

    fn authenticate(&self, _reason: &str) -> Result<(), BiometricError> {
        Err(BiometricError::Unavailable("no biometric sensor in this build".to_string()))
    }
}

/// The outcome of an unlock attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum Unlock {
    /// Unlocked via the biometric sensor.
    Biometric,
    /// Unlocked via the PIN (biometrics off, unavailable, or declined).
    Pin,
    /// Neither path succeeded.
    Denied,
}

/// Gating policy for unlocking the wallet behind the app lock.
///
/// - If `biometric_enabled` and the sensor is available, attempt biometrics first; success →
///   [`Unlock::Biometric`].
/// - Otherwise (or on biometric failure) verify `pin` against `lock`; success → [`Unlock::Pin`].
/// - If neither succeeds → [`Unlock::Denied`].
///
/// The PIN is always honoured, so an unenrolled or failing sensor degrades gracefully instead of
/// locking the user out.
pub fn unlock(
    lock: &AppLock,
    authenticator: &dyn BiometricAuthenticator,
    biometric_enabled: bool,
    pin: &str,
) -> Unlock {
    if biometric_enabled
        && authenticator.availability() == BiometricAvailability::Available
        && authenticator.authenticate("Unlock your wallet").is_ok()
    {
        return Unlock::Biometric;
    }
    if lock.verify(pin) {
        Unlock::Pin
    } else {
        Unlock::Denied
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock() -> AppLock {
        AppLock::new("1234").unwrap()
    }

    struct AlwaysAvailable {
        accepts: bool,
    }
    impl BiometricAuthenticator for AlwaysAvailable {
        fn availability(&self) -> BiometricAvailability {
            BiometricAvailability::Available
        }
        fn authenticate(&self, _reason: &str) -> Result<(), BiometricError> {
            if self.accepts { Ok(()) } else { Err(BiometricError::Failed) }
        }
    }

    #[test]
    fn unsupported_backend_is_unavailable_and_errors() {
        let backend = UnsupportedBiometric;
        assert!(matches!(backend.availability(), BiometricAvailability::Unavailable(_)));
        assert!(backend.authenticate("x").is_err());
    }

    #[test]
    fn falls_back_to_pin_when_biometric_unavailable() {
        // Even with biometrics "enabled", the unsupported backend forces the PIN path.
        let outcome = unlock(&lock(), &UnsupportedBiometric, true, "1234");
        assert_eq!(outcome, Unlock::Pin);
    }

    #[test]
    fn biometric_unlocks_when_enabled_available_and_accepted() {
        let outcome = unlock(&lock(), &AlwaysAvailable { accepts: true }, true, "wrong-pin");
        assert_eq!(outcome, Unlock::Biometric);
    }

    #[test]
    fn declined_biometric_falls_back_to_pin() {
        // Sensor present but rejects; a correct PIN still unlocks.
        let outcome = unlock(&lock(), &AlwaysAvailable { accepts: false }, true, "1234");
        assert_eq!(outcome, Unlock::Pin);
    }

    #[test]
    fn disabled_biometric_uses_pin_and_wrong_pin_is_denied() {
        let available = AlwaysAvailable { accepts: true };
        // Biometrics disabled → sensor never consulted; wrong PIN is denied.
        assert_eq!(unlock(&lock(), &available, false, "nope"), Unlock::Denied);
        assert_eq!(unlock(&lock(), &available, false, "1234"), Unlock::Pin);
    }
}
