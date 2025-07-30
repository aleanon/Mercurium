use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to establish app directory, source: {0}")]
    UnableToEstablishDirectory(std::io::Error),
    #[error("Unable to create app directory, source: {0}")]
    UnableToCreateDirectory(std::io::Error),
}

trait AppPath: Sized {
    type Network;

    fn new() -> Result<Self, Error>;

    fn app_directory(&self) -> PathBuf;

    fn config_directory(&self) -> PathBuf;

    fn settings_path(&self) -> PathBuf;

    fn db_directory(&self) -> PathBuf;

    fn db_path(&self, network: Self::Network) -> PathBuf;

    fn icons_directory(&self) -> PathBuf;

    fn icon_cache(&self, network: Self::Network) -> PathBuf;

    fn app_directory_ref(&self) -> &Box<Path>;

    fn config_directory_ref(&self) -> &Box<Path>;

    fn settings_path_ref(&self) -> &Box<Path>;

    fn db_directory_ref(&self) -> &Box<Path>;

    fn db_path_ref(&self, network: Self::Network) -> &Box<Path>;

    fn icons_directory_ref(&self) -> &Box<Path>;

    fn icon_cache_ref(&self, network: Self::Network) -> &Box<Path>;
}
