use types::{AppError, AppSettings};

/// Port for persisting the application settings (a small JSON document on disk, distinct from the
/// SQLite wallet data). Backed by [`crate::JsonSettingsStore`].
pub trait SettingsStore {
    /// Load the stored settings, returning defaults if none exist or they cannot be read.
    fn load(&self) -> AppSettings;

    /// Persist the settings.
    fn save(&self, settings: &AppSettings) -> Result<(), AppError>;
}
