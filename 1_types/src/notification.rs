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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_returns_inner_text_or_empty() {
        assert_eq!(Notification::Info("hi".to_string()).message(), "hi");
        assert_eq!(Notification::None.message(), "");
    }

    #[test]
    fn take_message_extracts_and_resets_to_none() {
        let mut notification = Notification::Warn("careful".to_string());
        assert_eq!(notification.take_message(), Some("careful".to_string()));
        assert!(matches!(notification, Notification::None));
        assert_eq!(notification.take_message(), None);
    }

    #[test]
    fn display_prefixes_by_severity() {
        assert_eq!(format!("{}", Notification::Success("ok".to_string())), "Success: ok");
        assert_eq!(format!("{}", Notification::Info("i".to_string())), "Info: i");
        assert_eq!(format!("{}", Notification::Warn("w".to_string())), "Warning: w");
        assert_eq!(format!("{}", Notification::Danger("d".to_string())), "Danger: d");
        assert_eq!(format!("{}", Notification::None), "");
    }

    #[test]
    fn default_is_none() {
        assert!(matches!(Notification::default(), Notification::None));
    }
}
