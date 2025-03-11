use types::crypto::Password;

use crate::app_state::WalletState;

use super::{unlocked::Unlocked, InnerWallet};

pub enum LoginError {
    IncorrectPassword,
    MaxAttemptsReached,
}

pub enum LoginResponse {
    Success(InnerWallet<Unlocked>),
    Failed(InnerWallet<Locked>, LoginError)
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

impl InnerWallet<Locked> {
    pub async fn login_with_password(self, password: Password) -> LoginResponse {
        if self.state.attempts >= self.max_login_attempts() {
            return LoginResponse::Failed(self, LoginError::MaxAttemptsReached)
        }

        match handles::wallet::perform_login_check(self.wallet_data.network(), &password).await {
            Ok(_) => LoginResponse::Success(InnerWallet { state: Unlocked, wallet_data: self.wallet_data}),
            Err(_) => {
                LoginResponse::Failed(self, LoginError::IncorrectPassword)
            }
        }
    }

    pub fn max_login_attempts(&self) -> usize {
        self.wallet_data.get_settings().max_login_attempts
    }
}