use deps::*;

use serde::{Deserialize, Serialize};

use crate::{Network, theme::Theme};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub max_login_attempts: usize,
    pub theme: Theme,
    pub network: Network,
}

impl AppSettings {
    const MAX_LOGIN_ATTEMPTS_DEFAULT: usize = 1000;

    pub fn new() -> Self {
        Self {
            max_login_attempts: Self::MAX_LOGIN_ATTEMPTS_DEFAULT,
            theme: Theme::Dark,
            network: Network::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_defaults() {
        let settings = AppSettings::new();
        assert_eq!(settings.max_login_attempts, 1000);
        assert_eq!(settings.network, Network::Mainnet);
    }

    #[test]
    fn serde_roundtrip_preserves_values() {
        let settings = AppSettings::new();
        let json = deps::serde_json::to_string(&settings).unwrap();
        let restored: AppSettings = deps::serde_json::from_str(&json).unwrap();
        assert_eq!(restored.max_login_attempts, settings.max_login_attempts);
        assert_eq!(restored.network, settings.network);
    }
}
