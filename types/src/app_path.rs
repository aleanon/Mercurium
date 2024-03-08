use std::path::PathBuf;

use thiserror::Error;


#[derive(Error, Debug)]
pub enum AppPathError {
    #[error("Unable to find app directory, source: {0}")]
    UnableToEstablishDirectory(std::io::Error),
    #[error("Unable to create app directory, source: {0}")]
    UnableToCreateDirectory(std::io::Error),
}

#[derive(Debug)]
pub struct AppPath {
    app_directory: PathBuf,
    db_directory: PathBuf,
    db_path: PathBuf,
    icons_directory: PathBuf,
}

impl AppPath {
    const STORE_DIRECTORY: &'static str = "database";
    const STORE_FILE_NAME: &'static str = "appdata";
    const STORE_EXTENSION: &'static str = "db";
    const ICON_DIRECTORY: &'static str = "icons";

    #[cfg(windows)]
    pub fn new() -> Result<Self, AppPathError> {

        match std::env::var_os("LOCALAPPDATA") {
            Some(path) => {

                let mut app_directory = std::path::PathBuf::from(path);
                app_directory.push("RaVault");
                let mut db_directory = app_directory.clone();
                db_directory.push(Self::STORE_DIRECTORY);
                let mut db_path = db_directory.clone();
                db_path.push(Self::STORE_FILE_NAME);
                db_path.set_extension(Self::STORE_EXTENSION);
                let mut icons_directory = app_directory.clone();
                icons_directory.push(Self::ICON_DIRECTORY);

                Ok(AppPath {
                    app_directory,
                    db_directory,
                    db_path,
                    icons_directory,
                })
            }
            None => {
                let app_directory = std::env::current_exe()
                    .map_err(|err| AppPathError::UnableToEstablishDirectory(err))?;

                let mut db_directory = app_directory.clone();
                db_directory.push(Self::STORE_DIRECTORY);
                let mut db_path = db_directory.clone();
                db_path.push(Self::STORE_FILE_NAME);
                db_path.set_extension(Self::STORE_EXTENSION);
                let mut icons_directory = app_directory.clone();
                icons_directory.push(Self::ICON_DIRECTORY);

                Ok(AppPath {
                    app_directory,
                    db_directory,
                    db_path,
                    icons_directory,
                })
            }
        }
    }

    pub fn create_directories_if_not_exists(self) -> Result<Self, AppPathError> {
        if !self.db_directory.exists() {

            std::fs::DirBuilder::new()
                .recursive(true)
                .create(&self.db_directory)
                .map_err(|err| {
                    AppPathError::UnableToCreateDirectory(err)
                })?;

        }

        if !self.icons_directory().exists() {
            std::fs::DirBuilder::new()
                .create(&self.icons_directory)
                .map_err(|err| {
                    AppPathError::UnableToCreateDirectory(err)
                })?;
        }

        Ok(self)
    }

    pub fn app_directory(self) -> PathBuf {
        self.app_directory
    }

    pub fn db_directory(self) -> PathBuf {
        self.db_directory
    }

    pub fn db_path(self) -> PathBuf {
        self.db_path
    }

    pub fn app_directory_ref(&self) -> &PathBuf {
        &self.app_directory
    }

    pub fn db_directory_ref(&self) -> &PathBuf {
        &self.db_directory
    }

    pub fn db_path_ref(&self) -> &PathBuf {
        &self.db_path
    }

    pub fn icons_directory(&self) -> &PathBuf {
        &self.icons_directory
    }
}
