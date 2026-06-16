//! JSON-file-backed [`SettingsStore`] adapter. Moved here from `handles::app_settings`.

use deps::*;

use std::fs::File;
use std::io::BufReader;

use types::{AppError, AppPath, AppSettings, Notification};

use crate::ports::settings_store::SettingsStore;

/// Stores the application settings as a pretty-printed JSON file at the app settings path.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonSettingsStore;

impl SettingsStore for JsonSettingsStore {
    fn load(&self) -> AppSettings {
        match File::open(AppPath::get().settings_path_ref()) {
            Ok(file) => serde_json::from_reader::<_, AppSettings>(BufReader::new(file))
                .unwrap_or(AppSettings::new()),
            Err(_) => AppSettings::new(),
        }
    }

    fn save(&self, settings: &AppSettings) -> Result<(), AppError> {
        let file = File::create(AppPath::get().settings_path_ref()).map_err(|err| {
            AppError::NonFatal(Notification::Warn(format!(
                "Unable to get app settings file: {err}"
            )))
        })?;
        serde_json::to_writer_pretty(file, settings).map_err(|err| {
            AppError::NonFatal(Notification::Warn(format!(
                "Unable to write app settings to file: {err}"
            )))
        })
    }
}
