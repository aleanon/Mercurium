use deps_two::*;

use thiserror::Error;

use crate::notification::Notification;

#[derive(Debug, Clone, Error)]
pub enum AppError {
    #[error("Fatal error occured, {0}")]
    Fatal(String),
    #[error("Non fatal error occured, {0}")]
    NonFatal(Notification),
    #[error("Ignoring Error")]
    Ignore,
}

#[derive(Debug, Error, Clone)]
#[error("{0}")]
pub struct ErrorString(pub String);


impl From<()> for AppError {
    fn from(_value: ()) -> Self {
        Self::Ignore
    }
}