use serde::{Deserialize, Serialize};

use crate::{theme::Theme, Network};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: Theme,
    pub network: Network,
}

impl AppSettings {
    pub fn new() -> Self {
        Self {
            theme: Theme::Dark,
            network: Network::Mainnet,
        }
    }
}
