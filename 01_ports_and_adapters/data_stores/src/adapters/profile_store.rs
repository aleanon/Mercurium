//! JSON-file-backed [`ProfileStore`] adapter for the live wallet profile.

use deps::*;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use types::{AppError, AppPathInner, Notification, Profile};

use crate::ports::profile_store::ProfileStore;

/// Stores the profile as pretty-printed JSON at `<config>/profile.json`. Holds the injected
/// [`AppPathInner`] (no path global).
#[derive(Debug, Clone)]
pub struct JsonProfileStore {
    paths: Arc<AppPathInner>,
}

impl JsonProfileStore {
    pub fn new(paths: Arc<AppPathInner>) -> Self {
        Self { paths }
    }

    fn path(&self) -> PathBuf {
        let mut path = self.paths.config_directory();
        path.push("profile.json");
        path
    }
}

impl ProfileStore for JsonProfileStore {
    fn load(&self) -> Profile {
        load_from(self.path()).unwrap_or_default()
    }

    fn save(&self, profile: &Profile) -> Result<(), AppError> {
        save_to(profile, self.path())
    }
}

/// Writes a profile as JSON to `path`.
pub fn save_to(profile: &Profile, path: impl AsRef<Path>) -> Result<(), AppError> {
    let json = serde_json::to_vec_pretty(profile).map_err(|err| {
        AppError::NonFatal(Notification::Warn(format!("Failed to serialize profile: {err}")))
    })?;
    std::fs::write(path, json)
        .map_err(|err| AppError::NonFatal(Notification::Warn(format!("Failed to write profile: {err}"))))
}

/// Reads a profile from `path`, returning `None` if it doesn't exist or can't be parsed.
pub fn load_from(path: impl AsRef<Path>) -> Option<Profile> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{AuthorizedDapp, GatewayConfig};

    fn sample() -> Profile {
        let mut profile = Profile::new();
        profile.gateways.switch_to(GatewayConfig::stokenet());
        profile.upsert_authorized_dapp(AuthorizedDapp {
            dapp_definition_address: "account_rdx_dapp".to_string(),
            display_name: Some("X".to_string()),
            origin: "https://x".to_string(),
            authorized_personas: vec![],
        });
        profile
    }

    #[test]
    fn save_to_then_load_from_roundtrips() {
        let profile = sample();
        let mut path = std::env::temp_dir();
        path.push(format!("mercurium_profile_store_test_{}.json", std::process::id()));
        save_to(&profile, &path).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(profile, loaded);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_from_missing_file_is_none() {
        assert!(load_from("/nonexistent/path/mercurium_profile.json").is_none());
    }
}
