
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Fatal error occured, {0}")]
    Fatal(Box<dyn std::error::Error>),
    #[error("Non fatal error occured, {0}")]
    NonFatal(Box<dyn std::error::Error>),
}