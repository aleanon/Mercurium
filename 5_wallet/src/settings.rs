use std::{fs::File, io::BufReader};

use deps::{
    serde::{Deserialize, Serialize},
    serde_json,
};
use types::{AppPath, Network};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub network: Network,
    pub max_login_attempts: usize,
}

impl Settings {
    const DEFAULT_MAX_LOGIN_ATTEMPTS: usize = 100;

    pub fn new() -> Self {
        Self {
            network: Network::default(),
            max_login_attempts: Self::DEFAULT_MAX_LOGIN_ATTEMPTS,
        }
    }

    pub fn load_from_disk_or_default() -> Self {
        match File::open(AppPath::get().settings_path_ref()) {
            Ok(file) => {
                let content = BufReader::new(file);
                serde_json::from_reader::<_, Self>(content).unwrap_or(Self::new())
            }
            Err(_) => Self::new(),
        }
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn set_network(&mut self, network: Network) {
        self.network = network;
    }

    pub fn max_login_attempts(&self) -> usize {
        self.max_login_attempts
    }

    pub fn set_max_login_attempts(&mut self, max_login_attempts: usize) {
        self.max_login_attempts = max_login_attempts;
    }
}
