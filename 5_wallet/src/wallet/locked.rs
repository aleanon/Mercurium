use deps::*;

use thiserror::Error;
use types::{crypto::Password, AppSettings};

use crate::{wallet::WalletState, WalletData};

use super::{unlocked::Unlocked, Wallet};

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Incorrect Password")]
    IncorrectPassword,
    #[error("Max login attempts reached")]
    MaxAttemptsReached,
}

pub enum LoginResponse {
    Success(Wallet<Unlocked>, bool),
    Failed(Wallet<Locked>, LoginError)
}

#[derive(Debug, Clone)]
pub struct Locked {
    attempts: usize,
    is_initial_login: bool,
}

impl Locked {
    pub fn new(is_initial_login: bool) -> Self {
        Self { attempts: 0, is_initial_login }
    }
}

impl WalletState for Locked{}

impl Wallet<Locked> {
    pub async fn login_with_password(self, password: Password) -> LoginResponse {
        if self.state.attempts >= self.max_login_attempts() {
            return LoginResponse::Failed(self, LoginError::MaxAttemptsReached)
        }

        
        match handles::wallet::perform_login_check(self.wallet_data.network(), &password).await {
            Ok(_) => LoginResponse::Success(Wallet { state: Unlocked, wallet_data: self.wallet_data}, self.state.is_initial_login),
            Err(_) => {
                LoginResponse::Failed(self, LoginError::IncorrectPassword)
            }
        }
    }

    pub fn max_login_attempts(&self) -> usize {
        self.wallet_data.get_settings().max_login_attempts
    }

    pub fn wallet_data_mut(&mut self) -> &mut WalletData {
        &mut self.wallet_data
    }

    pub fn is_initial_login(&self) -> bool {
        self.state.is_initial_login
    }
}

impl Default for Wallet<Locked> {
    fn default() -> Self {
        Self::new(Locked::new(true), WalletData::new(AppSettings::new()))
    }
}