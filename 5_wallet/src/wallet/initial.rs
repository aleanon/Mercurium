mod setup_tasks;


use std::sync::Arc;

use setup_tasks::SetupTasks;
use thiserror::Error;
use tokio::sync::Mutex;
use types::{crypto::{bip39::{self, Language, Mnemonic}, CryptoError, Password, SeedPhrase}, Account, AccountSummary, TaskResponse};

use crate::app_state::WalletState;

use super::InnerWallet;

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("Invalid seed phrase: {0}")]
    InvalidSeedPhrase(#[from] bip39::ErrorKind),
    #[error("Unable to generate key and salt")]
    UnableToGenerateKeyAndSalt(#[from] CryptoError),
}


pub struct Initial {
    pub mnemonic: Option<(Mnemonic, u8)>,
    pub seed_password: Option<Password>,
    pub password: Option<(Password, u8)>,
    pub accounts: Vec<Account>,
    pub tasks: SetupTasks,
}

impl Initial {
    pub fn new() -> Self {
        Self {
            mnemonic: None,
            seed_password: None,
            password: None,
            accounts: Vec::new(),
            tasks: SetupTasks::new()
        }
    }
}

impl WalletState for Initial{}


impl InnerWallet<Initial> {
    pub fn set_seed_phrase_and_password(&mut self, seed_phrase: SeedPhrase, seed_password: Option<Password>) -> Result<(), SetupError> {
        self.state.seed_password = seed_password.clone();

        let mnemonic = Mnemonic::from_phrase(seed_phrase.phrase().as_str(), Language::English)?;
        let Some(task_id) = self.set_mnemonic(mnemonic.clone()) else {
            return Ok(())
        };
        let network = self.wallet_data.settings.network;
        
        self.state.tasks.create_and_update_accounts(task_id, mnemonic, seed_password, network);

        Ok(())
    }

    fn set_mnemonic(&mut self, mnemonic: Mnemonic) -> Option<u8> {
        let task_id;
        if let Some((old_mnemonic, id)) = self.state.mnemonic.as_mut() {
            if mnemonic.phrase() == old_mnemonic.phrase() {return None}
            *old_mnemonic = mnemonic.clone();
            *id += 1;
            task_id = *id;
        } else {
            self.state.mnemonic = Some((mnemonic.clone(), 1));
            task_id = 1;
        }

        Some(task_id)
    }

 
    pub fn set_password(&mut self, password: Password) {
        let task_id:u8;
        if let Some((old_password, id)) = self.state.password.as_mut() {
            if old_password.as_str() == password.as_str() {return}
            *old_password = old_password.clone();
            *id += 1;
            task_id = *id;
        } else {
            self.state.password = Some((password.clone(), 1));
            task_id = 1;
        }
        
        self.state.tasks.create_encryption_keys(task_id, password);
    }

    pub fn set_accounts(&mut self, accounts: Vec<Account>) {
        self.state.accounts = accounts;
    }

    pub fn seed_phrase(&self) -> Option<&str> {
        self.state.mnemonic
            .as_ref()
            .and_then(|(mnemonic, _)| Some(mnemonic.phrase()))
    }

    pub fn seed_password(&self) -> Option<&str> {
        self.state.seed_password.as_ref()
            .map(|p| p.as_str())
    }

    pub fn password(&self) -> Option<&str> {
        self.state.password.as_ref()
            .map(|(p, _)| p.as_str())
    }

    pub fn created_accounts(&self) -> Arc<Mutex<TaskResponse<Vec<(Account, AccountSummary)>>>> {
        self.state.tasks.accounts.clone()
    }

    pub fn selected_accounts(&self) -> Vec<Account> {
        self.state.accounts.clone()
    }
}