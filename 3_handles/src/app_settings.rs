use deps_two::*;

use std::fs::File;
use std::io::BufReader;

use types::{AppError, AppPath, AppSettings, Notification};

pub fn get_app_settings() -> AppSettings {
    match File::open(AppPath::get().settings_path_ref()) {
        Ok(file) => {
            let content = BufReader::new(file);
            serde_json::from_reader::<_, AppSettings>(content).unwrap_or(AppSettings::new())
        }
        Err(_) => AppSettings::new(),
    }

}

pub fn save_app_settings(app_settings: AppSettings) -> Result<(), AppError> {
    let file = File::create(AppPath::get().settings_path_ref()).map_err(|err| {
        AppError::NonFatal(Notification::Warn(format!(
            "Unable to get app settings file: {err}"
        )))
    })?;
    serde_json::to_writer_pretty(file, &app_settings).map_err(|err| {
        AppError::NonFatal(Notification::Warn(format!(
            "Unable to write app settings to file: {err}"
        )))
    })
}
