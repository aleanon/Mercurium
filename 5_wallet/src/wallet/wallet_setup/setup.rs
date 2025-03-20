use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use types::{address::ResourceAddress, collections::AccountsUpdate, crypto::{bip39::Mnemonic, Password}, Account, Network, UnwrapUnreachable};

use crate::{app_state::WalletState, wallet_keys_and_salt::WalletEncryptionKeys};

use super::{setup_error::SetupError, task_manager::TaskManager};

#[derive(Clone)]
pub struct Setup {
    pub network: Network,
    pub mnemonic_with_password: Option<(Mnemonic, Option<Password>, u16)>,
    pub password: Option<(Password, u16)>,
    pub accounts: Vec<Account>,
    pub setup_tasks: Arc<TaskManager>,
}

impl Setup {
    pub fn new() -> Self {
        Self {
            network: Network::Mainnet, 
            mnemonic_with_password: None,
            password: None,
            accounts: Vec::new(),
            setup_tasks: Arc::new(TaskManager::new()),
        }
    }

    pub fn set_password(&mut self, new_password: Password) {
        match &mut self.password {
            Some((password, id)) => {
                if new_password.as_str() == password.as_str() {
                    return;
                }
                *password = new_password;
                *id += 1;
            }
            None => {
                self.password = Some((new_password, 1));
            }
        }
        let (password, id) = self.password.as_ref().unwrap_unreachable("Called unwrap on password, but no password was supplied");

        let task_manager = self.setup_tasks.clone();
        let password = password.clone();
        let id = *id;
        tokio::spawn(async move {
            task_manager.run_task_create_encryption_keys(id, password).await;
        });
    }

    pub fn set_mnemonic_and_password(&mut self, new_mnemonic: Mnemonic, new_seed_password: Option<Password>) {
        match &mut self.mnemonic_with_password {
            Some((mnemonic, seed_password, id)) => {
                if new_mnemonic.phrase() == mnemonic.phrase() && new_seed_password.as_ref() == seed_password.as_ref() {
                    return;
                }
                *mnemonic = new_mnemonic;
                *seed_password = new_seed_password;
                *id += 1;
            }
            None => {
                self.mnemonic_with_password = Some((new_mnemonic, new_seed_password, 1));
            }
        };

        let (mnemonic, seed_password, id) = self.mnemonic_with_password.as_ref()
            .unwrap_unreachable("Called unwrap on mnemonic with password when non where supplied");

        let task_manager = self.setup_tasks.clone();
        let network = self.network;
        let mnemonic = mnemonic.clone();
        let seed_password = seed_password.clone();
        let id = *id;
        tokio::spawn(async move {
            task_manager.run_task_create_and_update_accounts(id, mnemonic, seed_password, network).await;
        });
    }

    pub async fn get_keys_with_salt(&self) -> Result<WalletEncryptionKeys, SetupError> {
        if let None = &self.password {
            return Err(SetupError::NoPasswordProvided)
        };
        
        self.setup_tasks.get_wallet_encryption_keys().await
    }

    pub async fn get_updated_accounts(&self) -> Result<AccountsUpdate, SetupError> {
        if let None = &self.mnemonic_with_password {
            return Err(SetupError::NoMnemonicProvided)
        };

        self.setup_tasks.get_accounts_update().await
    }

    pub fn get_mnemonic(&self) -> Option<&Mnemonic> {
        self.mnemonic_with_password.as_ref().map(|(mnemonic, _, _)| mnemonic)
    }

    pub fn get_password(&self) -> Option<&Password> {
        self.password.as_ref().map(|(pw,_)| pw)
    }

    pub fn get_seed_password(&self) -> Option<&str> {
        self.mnemonic_with_password.as_ref().and_then(|(_, pw,_)| {
            pw.as_ref().map(|pw| pw.as_str())
        })
    }

    pub async fn get_icons(&self) -> HashMap<ResourceAddress, Bytes> {
        self.setup_tasks.get_icons_data().await.unwrap_or_default()
    }
}

impl WalletState for Setup{}