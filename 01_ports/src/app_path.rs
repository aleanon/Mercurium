use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to establish app directory, source: {0}")]
    UnableToEstablishDirectory(std::io::Error),
    #[error("Unable to create app directory, source: {0}")]
    UnableToCreateDirectory(std::io::Error),
}
