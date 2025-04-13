use deps::*;

use serde::{Deserialize, Serialize};

use crate::{theme::Theme, Network};

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
