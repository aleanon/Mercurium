use deps::{debug_print::debug_println, *};

use std::{collections::HashMap, sync::Arc};

use types::{address::ResourceAddress, collections::AccountsUpdate, crypto::{bip39::{Language, Mnemonic, MnemonicType}, Password}, Account, AppPath, AppSettings, Network, UnwrapUnreachable};

use crate::{wallet::WalletState, wallet_encryption_keys::WalletEncryptionKeys, Unlocked, Wallet, WalletData};

use super::{setup_error::SetupError, task_manager::TaskManager};

#[derive(Debug, Clone)]
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
            network: Network::default(), 
            mnemonic_with_password: None,
            password: None,
            accounts: Vec::new(),
            setup_tasks: Arc::new(TaskManager::new()),
        }
    }

    pub fn reset(&mut self) {
        self.network = Network::default();
        self.mnemonic_with_password = None;
        self.password = None;
        self.accounts.clear();
        self.setup_tasks = Arc::new(TaskManager::new());
    }

    pub fn create_random_seed_phrase(&mut self) {
        let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);
        let account = handles::wallet::create_account_from_mnemonic(&mnemonic, None, 0, 0, "Initial Account".to_string(), self.network);
        self.mnemonic_with_password = Some((mnemonic, None, 1));
        self.accounts.push(account);
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

    pub fn set_seed_password(&mut self, new_seed_password: Password) {
        let Some((_, seed_password, id)) = &mut self.mnemonic_with_password  else {
            return;
        };
        if Some(&new_seed_password) == seed_password.as_ref() {
            return;
        }
        *seed_password = Some(new_seed_password);
        *id += 1;
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

    pub async fn get_icons(&self) -> HashMap<ResourceAddress, (Vec<u8>, Vec<u8>)> {
        self.setup_tasks.get_icons_data().await.unwrap_or_default()
    }


    pub async fn finalize_setup(mut self) -> Result<Wallet<Unlocked>, SetupError> {
        let network = Network::default();
        let wallet_keys = self.get_keys_with_salt().await?;
        debug_println!("Wallet keys successfully fetched");

        let password_hash =  self.get_password().ok_or(SetupError::NoPasswordProvided)?
            .derive_db_encryption_key_hash_from_salt(wallet_keys.db_key_salt.salt());
        debug_println!("Password hashed");

        AppPath::get().create_directories_if_not_exists()?;

        let db_key = wallet_keys.db_key_salt.key().clone();

        handles::wallet::create_new_wallet_with_accounts(
            self.get_mnemonic().ok_or(SetupError::NoMnemonicProvided)?,
            self.get_seed_password(),
            wallet_keys.db_key_salt,
            wallet_keys.mnemonic_key_salt,
            password_hash,
            &self.accounts,
            network,
        )
        .await
        .map_err(|_| SetupError::Unspecified)?;
        
        debug_println!("Wallet Created");

        let mut accounts_update = self.get_updated_accounts().await?;

        debug_println!("Fetched account updates");

        let mut wallet_data = WalletData::new(AppSettings::new());


        for mut account in std::mem::take(&mut self.accounts) {
            let Some(account_update) = accounts_update.account_updates.iter_mut()
                .find(|account_update|&account_update.account.address == &account.address) else {continue};

            let fungibles = std::mem::take(&mut account_update.fungibles);
            let fungibles = fungibles.into_values().collect();

            wallet_data.resource_data
                .fungibles
                .insert(account_update.account.address.clone(), fungibles);

            let non_fungibles = std::mem::take(&mut account_update.non_fungibles);
            let non_fungibles = non_fungibles.into_values().collect();

            wallet_data.resource_data
                .non_fungibles
                .insert(account_update.account.address.clone(), non_fungibles);

            let updated_account = std::mem::take(&mut account_update.account);
            account.transactions_last_updated = updated_account.transactions_last_updated;

            wallet_data.resource_data.accounts.insert(
                updated_account.address,
                account,
            );

        }

        debug_println!("Wallet data created");

        wallet_data.resource_data.resources = accounts_update.new_resources;
        wallet_data.save_resource_data_to_disk(db_key.clone()).await?;

        let icons = self.get_icons().await;

        debug_println!("Fetched icons");

        let (icons_small, icons_standard) = icons.into_iter()
            .map(|(address, (small, standard))| {
                ((address.clone(), small), (address, standard))
            })
            .unzip();
        
        debug_println!("Splitted icons");

        wallet_data.resource_data.set_resource_icons(icons_small).await;
        debug_println!("Saved icons to wallet data");

        wallet_data.save_resource_icons_to_disk(icons_standard, db_key.clone()).await?;


        debug_println!("Saved {} icons to disk", wallet_data.resource_data.resource_icons.len());
        
        Ok(Wallet { state: Unlocked::new(db_key), wallet_data })
        
    }
}

impl WalletState for Setup{}