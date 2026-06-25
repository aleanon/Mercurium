use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::app_path_inner::AppPathInner;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to establish app directory, source: {0}")]
    UnableToEstablishDirectory(std::io::Error),
    #[error("Unable to create app directory, source: {0}")]
    UnableToCreateDirectory(std::io::Error),
}

type Network = types::Network;

pub trait AppPath {
    fn get() -> &'static Self
    where
        Self: Sized;

    fn app_directory(&self) -> &Box<Path>;

    fn config_directory(&self) -> &Box<Path>;

    fn settings_path(&self) -> &Box<Path>;

    fn db_directory(&self) -> &Box<Path>;

    fn db_path(&self, network: Network) -> &Box<Path>;

    fn icons_directory(&self) -> &Box<Path>;

    fn icon_cache(&self, network: Network) -> &Box<Path>;
}
