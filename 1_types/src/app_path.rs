use deps_two::*;

use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use thiserror::Error;

use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable, Network};

static APP_PATH: Lazy<AppPathInner> = Lazy::new(|| {
    AppPathInner::new().unwrap_unreachable(debug_info!("Unable to establish application directory"))
});

#[derive(Error, Debug)]
pub enum AppPathError {
    #[error("Unable to find app directory, source: {0}")]
    UnableToEstablishDirectory(std::io::Error),
    #[error("Unable to create app directory, source: {0}")]
    UnableToCreateDirectory(std::io::Error),
}

#[derive(Debug)]
pub struct AppPathInner {
    app_directory: Box<Path>,
    app_settings_path: Box<Path>,
    db_directory: Box<Path>,
    mainnet_db_path: Box<Path>,
    stokenet_db_path: Box<Path>,
    icons_directory: Box<Path>,
    mainnet_icon_cache_path: Box<Path>,
    stokenet_icon_cache_path: Box<Path>,
}

impl AppPathInner {
    pub const APP_DIRECTORY: &'static str = crate::consts::APPLICATION_NAME;
    const APP_SETTINGS_FILE_NAME: &'static str = "settings";
    const APP_SETTINGS_EXTENSION: &'static str = "json";
    const STORE_DIRECTORY: &'static str = "database";
    const STORE_MAINNET_FILE_NAME: &'static str = "mainnet";
    const STORE_STOKENET_FILE_NAME: &'static str = "stokenet";
    const DB_EXTENSION: &'static str = "db";
    const ICONS_DIRECTORY: &'static str = "icons";
    const ICONCASHE_MAINNET_FILE_NAME: &'static str = "iconcash_mainnet";
    const ICONCASHE_STOKENET_FILE_NAME: &'static str = "iconcash_stokenet";

    pub fn new() -> Result<Self, AppPathError> {
        let app_directory = Self::get_application_root_directory()?;

        let mut app_settings_path = app_directory.clone();
        app_settings_path.push(Self::APP_SETTINGS_FILE_NAME);
        app_settings_path.set_extension(Self::APP_SETTINGS_EXTENSION);

        let mut db_directory = app_directory.clone();
        db_directory.push(Self::STORE_DIRECTORY);

        let mut mainnet_db_path = db_directory.clone();
        mainnet_db_path.push(Self::STORE_MAINNET_FILE_NAME);
        mainnet_db_path.set_extension(Self::DB_EXTENSION);

        let mut stokenet_db_path = db_directory.clone();
        stokenet_db_path.push(Self::STORE_STOKENET_FILE_NAME);
        stokenet_db_path.set_extension(Self::DB_EXTENSION);

        let mut icons_directory = app_directory.clone();
        icons_directory.push(Self::ICONS_DIRECTORY);

        let mut mainnet_icon_cache_path = icons_directory.clone();
        mainnet_icon_cache_path.push(Self::ICONCASHE_MAINNET_FILE_NAME);
        mainnet_icon_cache_path.set_extension(Self::DB_EXTENSION);

        let mut stokenet_icon_cache_path = icons_directory.clone();
        stokenet_icon_cache_path.push(Self::ICONCASHE_STOKENET_FILE_NAME);
        stokenet_icon_cache_path.set_extension(Self::DB_EXTENSION);

        Ok(Self {
            app_directory: app_directory.into_boxed_path(),
            app_settings_path: app_settings_path.into_boxed_path(),
            db_directory: db_directory.into_boxed_path(),
            mainnet_db_path: mainnet_db_path.into_boxed_path(),
            stokenet_db_path: stokenet_db_path.into_boxed_path(),
            icons_directory: icons_directory.into_boxed_path(),
            mainnet_icon_cache_path: mainnet_icon_cache_path.into_boxed_path(),
            stokenet_icon_cache_path: stokenet_icon_cache_path.into_boxed_path(),
        })
    }


    pub fn create_directories_if_not_exists(&self) -> Result<&Self, AppPathError> {
        if !self.db_directory.exists() {
            std::fs::DirBuilder::new()
                .recursive(true)
                .create(&self.db_directory)
                .map_err(|err| AppPathError::UnableToCreateDirectory(err))?;
        }

        if !self.icons_directory.exists() {
            std::fs::DirBuilder::new()
                .create(&self.icons_directory)
                .map_err(|err| AppPathError::UnableToCreateDirectory(err))?;
        }

        Ok(self)
    }

    pub fn app_directory(&self) -> PathBuf {
        self.app_directory.to_path_buf()
    }

    pub fn settings_path(&self) -> PathBuf {
        self.app_settings_path.to_path_buf()
    }

    pub fn db_directory(&self) -> PathBuf {
        self.db_directory.to_path_buf()
    }

    pub fn db_path(&self, network: Network) -> PathBuf {
        match network {
            Network::Mainnet => self.mainnet_db_path.to_path_buf(),
            Network::Stokenet => self.stokenet_db_path.to_path_buf(),
        }
    }

    pub fn icons_directory(&self) -> PathBuf {
        self.icons_directory.to_path_buf()
    }

    pub fn icon_cache(&self, network: Network) -> PathBuf {
        match network {
            Network::Mainnet => self.mainnet_icon_cache_path.to_path_buf(),
            Network::Stokenet => self.stokenet_icon_cache_path.to_path_buf(),
        }
    }

    pub fn app_directory_ref(&self) -> &Box<Path> {
        &self.app_directory
    }

    pub fn settings_path_ref(&self) -> &Box<Path> {
        &self.app_settings_path
    }

    pub fn db_directory_ref(&self) -> &Box<Path> {
        &self.db_directory
    }

    pub fn db_path_ref(&self, network: Network) -> &Box<Path> {
        match network {
            Network::Mainnet => &self.mainnet_db_path,
            Network::Stokenet => &self.stokenet_db_path,
        }
    }

    pub fn icons_directory_ref(&self) -> &Box<Path> {
        &self.icons_directory
    }

    pub fn icon_cache_ref(&self, network: Network) -> &Box<Path> {
        match network {
            Network::Mainnet => &self.mainnet_icon_cache_path,
            Network::Stokenet => &self.stokenet_icon_cache_path,
        }
    }

    #[cfg(windows)]
    pub fn get_application_root_directory() -> Result<PathBuf, AppPathError> {
        match std::env::var_os("LOCALAPPDATA") {
            Some(path) => {
                let mut app_directory = std::path::PathBuf::from(path);
                app_directory.push(Self::APP_DIRECTORY);
                Ok(app_directory)
            }
            None => {
                let mut app_directory = std::env::current_exe()
                    .map_err(|err| AppPathError::UnableToEstablishDirectory(err))?;
                app_directory.push(Self::APP_DIRECTORY);
                Ok(app_directory)
            }
        }
    }
}

// // The use of this type required the PATH static to be set at program startup and never Uninitialized.
pub struct AppPath;

impl AppPath {
    pub fn get() -> &'static AppPathInner {
        &APP_PATH
    }
}
