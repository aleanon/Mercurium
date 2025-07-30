use deps::*;

use std::{
    cell::OnceCell,
    path::{Path, PathBuf},
};

use once_cell::sync::Lazy;
use thiserror::Error;

use crate::{Network, debug_info, unwrap_unreachable::UnwrapUnreachable};

#[derive(Error, Debug)]
pub enum AppPathError {
    #[error("Unable to find app directory, source: {0}")]
    UnableToEstablishDirectory(std::io::Error),
    #[error("Unable to create app directory, source: {0}")]
    UnableToCreateDirectory(std::io::Error),
}

#[derive(Debug)]
pub(crate) struct AppPathInner {
    pub app_directory: Box<Path>,
    pub config_directory: Box<Path>,
    pub app_settings_path: Box<Path>,
    pub db_directory: Box<Path>,
    pub mainnet_db_path: Box<Path>,
    pub stokenet_db_path: Box<Path>,
    pub icons_directory: Box<Path>,
    pub mainnet_icon_cache_path: Box<Path>,
    pub stokenet_icon_cache_path: Box<Path>,
}

impl AppPathInner {
    pub const APP_NAME: &'static str = crate::consts::APPLICATION_NAME;
    const CONFIG_DIRECTORY: &'static str = "config";
    const APP_SETTINGS_FILE_NAME: &'static str = "settings";
    const APP_SETTINGS_EXTENSION: &'static str = "json";
    const DATA_DIRECTORY: &'static str = "database";
    const STORE_MAINNET_FILE_NAME: &'static str = "mainnet";
    const STORE_STOKENET_FILE_NAME: &'static str = "stokenet";
    const DB_EXTENSION: &'static str = "db";
    const ICONS_DIRECTORY: &'static str = "icons";
    const ICONCASHE_MAINNET_FILE_NAME: &'static str = "iconcash_mainnet";
    const ICONCASHE_STOKENET_FILE_NAME: &'static str = "iconcash_stokenet";

    pub fn new(mut root_directory: PathBuf) -> Self {
        root_directory.push(Self::APP_NAME);

        let mut config_directory = root_directory.clone();
        config_directory.push(Self::CONFIG_DIRECTORY);

        let mut app_settings_path = config_directory.clone();
        app_settings_path.push(Self::APP_SETTINGS_FILE_NAME);
        app_settings_path.set_extension(Self::APP_SETTINGS_EXTENSION);

        let mut db_directory = root_directory.clone();
        db_directory.push(Self::DATA_DIRECTORY);

        let mut mainnet_db_path = db_directory.clone();
        mainnet_db_path.push(Self::STORE_MAINNET_FILE_NAME);
        mainnet_db_path.set_extension(Self::DB_EXTENSION);

        let mut stokenet_db_path = db_directory.clone();
        stokenet_db_path.push(Self::STORE_STOKENET_FILE_NAME);
        stokenet_db_path.set_extension(Self::DB_EXTENSION);

        let mut icons_directory = root_directory.clone();
        icons_directory.push(Self::ICONS_DIRECTORY);

        let mut mainnet_icon_cache_path = icons_directory.clone();
        mainnet_icon_cache_path.push(Self::ICONCASHE_MAINNET_FILE_NAME);
        mainnet_icon_cache_path.set_extension(Self::DB_EXTENSION);

        let mut stokenet_icon_cache_path = icons_directory.clone();
        stokenet_icon_cache_path.push(Self::ICONCASHE_STOKENET_FILE_NAME);
        stokenet_icon_cache_path.set_extension(Self::DB_EXTENSION);

        Ok(Self {
            app_directory: root_directory.into_boxed_path(),
            config_directory: config_directory.into_boxed_path(),
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
        if !self.config_directory.exists() {
            std::fs::DirBuilder::new()
                .create(&self.config_directory)
                .map_err(|err| AppPathError::UnableToCreateDirectory(err))?;
        }

        Ok(self)
    }
}
