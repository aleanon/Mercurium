use std::{fmt::Display, mem};

#[derive(Debug, Clone)]
pub enum Notification {
    None,
    Success(String),
    Info(String),
    Warn(String),
    Danger(String),
}

impl Notification {
    /// Takes the message if any and leaves Notification::None in its place
    pub fn take_message(&mut self) -> Option<String> {
        mem::replace(self, Self::None).get_string()
    }

    fn get_string(self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Success(message)
            | Self::Info(message)
            | Self::Warn(message)
            | Self::Danger(message) => Some(message),
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
