use types::{AppError, Profile};

/// Port for persisting the live (unencrypted) wallet [`Profile`] on disk, loaded at startup.
/// This is the working config; the *encrypted* backup is a separate concern (`profile_backup`
/// in the wallet crate).
pub trait ProfileStore {
    /// Loads the stored profile, returning a fresh default if none exists or it can't be read.
    fn load(&self) -> Profile;

    /// Persists the profile.
    fn save(&self, profile: &Profile) -> Result<(), AppError>;
}
