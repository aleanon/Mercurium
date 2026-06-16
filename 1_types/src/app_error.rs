use deps::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages() {
        assert_eq!(AppError::Fatal("boom".to_string()).to_string(), "Fatal error occured, boom");
        assert_eq!(AppError::Ignore.to_string(), "Ignoring Error");
        assert_eq!(
            AppError::NonFatal(Notification::Info("hi".to_string())).to_string(),
            "Non fatal error occured, Info: hi"
        );
    }

    #[test]
    fn from_unit_is_ignore() {
        let error: AppError = ().into();
        assert!(matches!(error, AppError::Ignore));
    }

    #[test]
    fn error_string_displays_inner() {
        assert_eq!(ErrorString("oops".to_string()).to_string(), "oops");
    }
}
