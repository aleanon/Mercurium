use types::crypto::Password;

use crate::app_state::WalletState;

use super::{unlocked::Unlocked, Wallet};

pub enum LoginError {
    IncorrectPassword,
    MaxAttemptsReached,
}

pub enum LoginResponse {
    Success(Wallet<Unlocked>),
    Failed(Wallet<Locked>, LoginError)
}

pub struct Locked {
    attempts: usize,
}

impl Locked {
    pub fn new() -> Self {
        Self { attempts: 0 }
    }
}

impl WalletState for Locked{}

impl Wallet<Locked> {
    async fn login_with_password(self, password: Password) -> LoginResponse {
        if self.state.attempts >= self.max_login_attempts() {
            return LoginResponse::Failed(self, LoginError::MaxAttemptsReached)
        }

        match handles::wallet::perform_login_check(self.wallet_data.network(), &password).await {
            Ok(_) => LoginResponse::Success(Wallet::new(Unlocked, self.wallet_data)),
            Err(_) => {
                LoginResponse::Failed(self, LoginError::IncorrectPassword)
            }
        }
    }

    fn max_login_attempts(&self) -> usize {
        self.wallet_data.get_settings().max_login_attempts()
    }
}