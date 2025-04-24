use deps::{debug_print::debug_println, *};
use store::{AppDataDb, DataBase, IconsDb};

use std::{collections::HashMap, sync::Arc};

use types::{crypto::Key, address::ResourceAddress, collections::AccountsUpdate, crypto::{bip39::{Language, Mnemonic, MnemonicType}, EncryptedMnemonic, HashedPassword, KeySaltPair, Password}, Account, AppError, AppPath, Network, UnwrapUnreachable};

use crate::{settings::Settings, wallet::{create_account_from_mnemonic, resource_data::ResourceData, WalletState}, wallet_encryption_keys::WalletEncryptionKeys, Unlocked, Wallet, WalletData};

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
        let account = create_account_from_mnemonic(&mnemonic, None, 0, 0, "Initial Account".to_string(), self.network);
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
            .unwrap_unreachable("Called unwrap on mnemonic with password when none where supplied");

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
        let settings = Settings::load_from_disk_or_default();
        let wallet_keys = self.get_keys_with_salt().await?;

        let password_hash =  self.get_password().ok_or(SetupError::NoPasswordProvided)?
            .derive_db_encryption_key_hash_from_salt(wallet_keys.db_key_salt.salt());

        AppPath::get().create_directories_if_not_exists()?;

        let db_key = wallet_keys.db_key_salt.key().clone();

        create_new_wallet_with_accounts(
            self.get_mnemonic().ok_or(SetupError::NoMnemonicProvided)?,
            self.get_seed_password(),
            wallet_keys.db_key_salt,
            wallet_keys.mnemonic_key_salt,
            password_hash,
            &self.accounts,
            settings.network,
        )
        .await
        .map_err(|_| SetupError::Unspecified)?;
        
        let mut wallet_data = WalletData::new(settings);

        save_updated_accounts_to_resource_data(
            std::mem::take(&mut self.accounts), 
            self.get_updated_accounts().await?, 
            Arc::make_mut(&mut wallet_data.resource_data)
        );

        save_icons_to_resource_data_and_disk(
            self.get_icons().await, 
            Arc::make_mut(&mut wallet_data.resource_data), 
            db_key.clone(), 
            wallet_data.settings.network
        ).await?;

        wallet_data.save_resource_data_to_disk(db_key.clone()).await?;

        debug_println!("Saved {} icons to disk", wallet_data.resource_data.resource_icons.len());
        
        Ok(Wallet { state: Unlocked::new(db_key), wallet_data })
        
    }
}

impl WalletState for Setup{}

/// Encrypts the mnemonic and stores it using the OS credentials system.
/// It also makes the initial creation of the database and stores the passed in accounts
pub async fn create_new_wallet_with_accounts(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    mut db_key_salt: KeySaltPair<DataBase>,
    mnemonic_key_salt: KeySaltPair<EncryptedMnemonic>,
    password_hash: HashedPassword,
    accounts: &[Account],
    network: Network,
) -> Result<(), AppError> {
    let encrypted_mnemonic = EncryptedMnemonic::new_with_key_and_salt(
        mnemonic,
        seed_password.unwrap_or(""),
        mnemonic_key_salt,
    )
    .map_err(|err| AppError::Fatal(err.to_string()))?;

    handles::credentials::store_encrypted_mnemonic(&encrypted_mnemonic)
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    handles::credentials::store_db_encryption_salt(db_key_salt.take_salt())?;

    let db = AppDataDb::load(network, db_key_salt.take_key())
        .await
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    db.upsert_password_hash(password_hash).await
        .map_err(|err| AppError::Fatal(err.to_string()))?;
    db.upsert_accounts(accounts.to_vec()).await.ok();

    Ok(())
}

fn save_updated_accounts_to_resource_data(accounts: Vec<Account>, mut accounts_update: AccountsUpdate, resource_data: &mut ResourceData) {
    for mut account in accounts {
        let Some(account_update) = accounts_update.account_updates.iter_mut()
            .find(|account_update|&account_update.account.address == &account.address) else {continue};

        let fungibles = std::mem::take(&mut account_update.fungibles);
        let fungibles = fungibles.into_values().collect();

        resource_data
            .fungibles
            .insert(account_update.account.address.clone(), fungibles);

        let non_fungibles = std::mem::take(&mut account_update.non_fungibles);
        let non_fungibles = non_fungibles.into_values().collect();

        resource_data
            .non_fungibles
            .insert(account_update.account.address.clone(), non_fungibles);

        let updated_account = std::mem::take(&mut account_update.account);
        account.transactions_last_updated = updated_account.transactions_last_updated;

        resource_data.accounts.insert(
            updated_account.address,
            account,
        );
    }
    resource_data.resources = accounts_update.new_resources;
}

async fn save_icons_to_resource_data_and_disk(icons: HashMap<ResourceAddress, (Vec<u8>, Vec<u8>)>, resource_data: &mut ResourceData, db_key: Key<DataBase>, network: Network) -> Result<(), SetupError> {                        
    let (icons_small, icons_standard) = icons.into_iter()
    .map(|(address, (small, standard))| {
        ((address.clone(), small), (address, standard))
    })
    .unzip();
            
    resource_data.set_resource_icons(icons_small).await;

    let db = IconsDb::get_or_init(network, db_key).await?;
    db.upsert_resource_icons(icons_standard).await?;
    Ok(())
}