use std::{fmt::Display, mem};

#[derive(Debug, Clone, Default)]
pub enum Notification {
    #[default]
    None,
    Success(String),
    Info(String),
    Warn(String),
    Danger(String),
}

impl Notification {
    /// Takes the message if any and leaves Notification::None in its place
    pub fn take_message(&mut self) -> Option<String> {
        mem::take(self).into_message()
    }

    fn into_message(self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Success(message)
            | Self::Info(message)
            | Self::Warn(message)
            | Self::Danger(message) => Some(message),
        }
    }
    
    pub fn message(&self) -> &str {
        match self {
            Self::None => "",
            Self::Success(message)
            | Self::Info(message)
            | Self::Warn(message)
            | Self::Danger(message) => message.as_str(),
        }
    }
}


impl Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Success(message) => write!(f, "Success: {}", message),
            Self::Info(message) => write!(f, "Info: {}", message),
            Self::Warn(message) => write!(f, "Warning: {}", message),
            Self::Danger(message) => write!(f, "Danger: {}", message),
        }
    }
}
