use std::cell::OnceCell;

use crate::{app_path_inner::AppPathInner, port::AppPath};

static APP_PATH_INNER: OnceCell<AppPathInner> = OnceCell::new();

pub struct LocalAppData(&'static AppPathInner);

impl LocalAppData {
    fn init() -> Self {
        let root_directory =
            get_local_app_data_directory().expect("Failed to get local app data directory");

        let app_path_inner = AppPathInner::new(root_directory);
        app_path_inner
            .create_directories_if_not_exists()
            .expect("Unable to create app directory");

        APP_PATH_INNER
            .set(app_path_inner)
            .expect("attempted to create App path inner twice");

        Self(APP_PATH_INNER.get().unwrap())
    }
}

impl AppPath for LocalAppData {
    type Network = types::Network;

    fn new() -> Self {
        match APP_PATH_INNER.get() {
            Some(app_path_inner) => Self(app_path_inner),
            None => Self::init(),
        }
    }

    fn app_directory(&self) -> &Box<std::path::Path> {
        &self.0.app_directory
    }

    fn config_directory(&self) -> &Box<std::path::Path> {
        &self.0.config_directory
    }

    fn db_directory(&self) -> &Box<std::path::Path> {
        &self.0.db_directory
    }

    fn db_path(&self, network: Self::Network) -> &Box<std::path::Path> {
        match network {
            Self::Network::Mainnet => &self.0.mainnet_db_path,
            Self::Network::Stokenet => &self.0.stokenet_db_path,
        }
    }

    fn icons_directory(&self) -> &Box<std::path::Path> {
        &self.0.icons_directory
    }

    fn icon_cache(&self, network: Self::Network) -> &Box<std::path::Path> {
        match network {
            Self::Network::Mainnet => &self.0.mainnet_icon_cache_path,
            Self::Network::Stokenet => &self.0.stokenet_icon_cache_path,
        }
    }

    fn settings_path(&self) -> &Box<std::path::Path> {
        &self.0.app_settings_path
    }
}

#[cfg(unix)]
pub fn get_local_app_data_directory() -> Result<PathBuf, AppPathError> {
    use std::io::ErrorKind;

    match std::env::var_os("XDG_DATA_HOME") {
        Some(path) => {
            let mut app_directory = std::path::PathBuf::from(path);
            Ok(app_directory)
        }
        None => {
            let app_directory = std::env::var_os("HOME").ok_or(
                AppPathError::UnableToEstablishDirectory(std::io::Error::from(ErrorKind::NotFound)),
            )?;
            let mut app_directory = std::path::PathBuf::from(app_directory);
            app_directory.push(".local");
            Ok(app_directory)
        }
    }
}

#[cfg(windows)]
pub fn get_local_app_data_directory() -> Result<PathBuf, AppPathError> {
    match std::env::var_os("LOCALAPPDATA") {
        Some(path) => {
            let mut app_directory = std::path::PathBuf::from(path);
            Ok(app_directory)
        }
        None => Err(AppPathError::UnableToEstablishDirectory(
            std::io::Error::from(std::io::ErrorKind::NotFound),
        )),
    }
}
