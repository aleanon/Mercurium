use std::sync::Arc;

use data_stores::{DataBase, IconsDb};
use secrets_store::SecretsStore;
use thiserror::Error;
use types::crypto::{Key, Password};

use crate::{WalletData, settings::Settings, wallet::WalletState};

use super::{Wallet, unlocked::Unlocked};

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Incorrect Password")]
    IncorrectPassword,
    #[error("Max login attempts reached")]
    MaxAttemptsReached,
    #[error("Unrecoverable error")]
    Unrecoverable,
}

pub enum LoginResponse {
    Success(Wallet<Unlocked>, bool),
    Failed(Wallet<Locked>, LoginError),
}

#[derive(Debug, Clone)]
pub struct Locked {
    attempts: usize,
    is_initial_login: bool,
}

impl Locked {
    pub fn new(is_initial_login: bool) -> Self {
        Self {
            attempts: 0,
            is_initial_login,
        }
    }
}

impl WalletState for Locked {}

impl Wallet<Locked> {
    pub async fn login_with_password(self, password: Password) -> LoginResponse {
        if self.state.attempts >= self.max_login_attempts() {
            return LoginResponse::Failed(self, LoginError::MaxAttemptsReached);
        }

        let Ok(salt) = self.wallet_data.env.secrets.get_db_encryption_salt() else {
            return LoginResponse::Failed(self, LoginError::Unrecoverable);
        };

        let key = Key::<DataBase>::new(password.as_str(), &salt);

        let mut wallet = match crate::wallet::login::perform_login_check(
            &self.wallet_data.env.secrets,
            self.wallet_data.settings.network,
            &password,
        )
        .await
        {
            // `perform_login_check` returns the opened DB handle; it lives in the Unlocked state.
            Ok(db) => Wallet {
                state: Unlocked::new(key, db),
                wallet_data: self.wallet_data,
            },
            Err(_) => return LoginResponse::Failed(self, LoginError::IncorrectPassword),
        };

        if self.state.is_initial_login {
            let Ok(icons_db) = IconsDb::get_or_init(
                wallet.wallet_data.settings.network,
                wallet.state.key.clone(),
            )
            .await
            else {
                return LoginResponse::Failed(
                    Wallet {
                        state: Locked::new(true),
                        wallet_data: wallet.wallet_data,
                    },
                    LoginError::Unrecoverable,
                );
            };

            let db = wallet.state.db.clone();
            let resources = Arc::make_mut(&mut wallet.wallet_data.resource_data);

            resources
                .load_resource_data_from_disk(&db, icons_db)
                .await
                .inspect_err(|err| eprintln!("Failed to load resource data: {err}"))
                .ok();
        }

        LoginResponse::Success(wallet, self.state.is_initial_login)
    }

    pub fn max_login_attempts(&self) -> usize {
        self.wallet_data.settings.max_login_attempts
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
        Self::new(
            Locked::new(true),
            WalletData::new(Settings::load_from_disk_or_default()),
        )
    }
}
