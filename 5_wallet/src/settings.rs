use types::Network;

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

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn max_login_attempts(&self) -> usize {
        self.max_login_attempts
    }
}