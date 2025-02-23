use std::{collections::HashMap, sync::{Arc, Mutex}};

use bytes::Bytes;
use store::DataBase;
use thiserror::Error;
use types::{address::ResourceAddress, collections::AccountsUpdate, crypto::{bip39::{self, Language, Mnemonic}, CryptoError, EncryptedMnemonic, Key, KeySaltPair, Password, Phrase}, Account, AccountSummary, TaskResponse};

use crate::{app_state::WalletState, wallet_keys_and_Salt::WalletEncryptionKeys};

use super::Wallet;

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("Invalid seed phrase: {0}")]
    InvalidSeedPhrase(#[from] bip39::ErrorKind),
    #[error("Unable to generate key and salt")]
    UnableToGenerateKeyAndSalt(#[from] CryptoError),
}

pub struct SetupTaskResponses {
    mnemonic_key_salt: Arc<TaskResponse<KeySaltPair<EncryptedMnemonic>>>,
    db_key_and_salt: Arc<TaskResponse<KeySaltPair<DataBase>>>,
    accounts: Arc<TaskResponse<Vec<(Account, AccountSummary)>>>,
    accounts_update: Arc<TaskResponse<AccountsUpdate>>,
    icons_data: Arc<TaskResponse<HashMap<ResourceAddress, Bytes>>>,
}

pub struct Initial {
    pub mnemonic: Option<Mnemonic>,
    pub seed_password: Option<Password>,
    pub password: Option<Password>,
    pub accounts: Vec<Account>,
    pub task_responses: SetupTaskResponses, 
}

impl WalletState for Initial{}


impl Wallet<Initial> {
    pub fn set_seed_phrase(&mut self, seed_phrase: Phrase, language: Language) -> Result<(), SetupError> {
        let mnemonic = Mnemonic::from_phrase(seed_phrase.as_str(), language)?;
        self.state.mnemonic = Some(mnemonic);

        Ok(())
    }

    pub fn set_seed_password(&mut self, seed_password: Password) {
        self.state.seed_password = Some(seed_password);
    }

    pub fn set_password(&mut self, password: Password) {
        self.state.password = Some(password);
    }

    pub fn set_accounts(&mut self, accounts: Vec<Account>) {
        self.state.accounts = accounts;
    }

    pub fn generate_key_and_salt(password: &Password) -> Result<WalletEncryptionKeys, SetupError> {
        Ok(WalletEncryptionKeys::new(password)?)
    }

}