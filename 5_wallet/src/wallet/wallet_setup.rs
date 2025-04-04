pub mod task_manager;
pub mod task_runner;
pub mod setup;
pub mod setup_error;


use std::sync::Arc;

use setup::Setup;
use setup_error::SetupError;
use store::IconsDb;
use task_manager::TaskManager;
use types::{crypto::{bip39::{Language, Mnemonic}, Password, Phrase}, Account, AppPath};


use super::{unlocked::Unlocked, Wallet};


impl Wallet<Setup> {
    pub fn set_seed_phrase_and_password(&mut self, seed_phrase: Phrase, seed_password: Option<Password>) -> Result<(), SetupError> {
        let mnemonic = Mnemonic::from_phrase(seed_phrase.as_str(), Language::English)?;

        self.state.set_mnemonic_and_password(mnemonic, seed_password);
        Ok(())
    }

    pub fn create_random_mnemonic(&mut self) {
        self.state.create_random_seed_phrase();
    }
 
    pub fn set_password(&mut self, password: Password) {
        self.state.set_password(password);
    }

    pub fn set_seed_password(&mut self, password: Password) {
        self.state.set_seed_password(password);
    }

    pub fn set_accounts(&mut self, accounts: Vec<Account>) {
        self.state.accounts = accounts;
    }

    pub fn task_manager(&self) -> Arc<TaskManager> {
        self.state.setup_tasks.clone()
    }

    pub fn seed_phrase(&self) -> Option<&str> {
        self.state.get_mnemonic().map(|m|m.phrase())
    }

    pub fn seed_password(&self) -> Option<&str> {
        self.state.get_seed_password()
    }

    pub fn password(&self) -> Option<&str> {
        self.state.get_password().map(|pw|pw.as_str())
    }

    pub fn selected_accounts(&self) -> Vec<Account> {
        self.state.accounts.clone()
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }

    pub fn get_setup(&self) -> Setup {
        self.state.clone()
    }

    pub fn finalize_setup(&mut self) -> impl Future<Output = Result<Wallet<Unlocked>, SetupError>> {
        let setup = self.state.clone();
        let network = self.wallet_data.settings.network;
        let mut wallet_data = self.wallet_data.clone();

        async move {
            let wallet_keys = setup.get_keys_with_salt().await?;

            let password_hash =  setup.get_password().ok_or(SetupError::NoPasswordProvided)?
                .derive_db_encryption_key_hash_from_salt(wallet_keys.db_key_salt.salt());

            AppPath::get().create_directories_if_not_exists()?;

            let db_key = wallet_keys.db_key_salt.key().clone();

            handles::wallet::create_new_wallet_with_accounts(
                setup.get_mnemonic().ok_or(SetupError::NoMnemonicProvided)?,
                setup.get_seed_password(),
                wallet_keys.db_key_salt,
                wallet_keys.mnemonic_key_salt,
                password_hash,
                &setup.accounts,
                network,
            )
            .await
            .map_err(|_| SetupError::Unspecified)?;

            IconsDb::load(network, db_key).await?;

            let accounts_update = setup.get_updated_accounts().await?;
 
            for account in &setup.accounts {
                for account_update in accounts_update.account_updates.clone() {
                    if account_update.account.address != account.address {continue};
                    
                    let fungibles = account_update.fungibles.into_values().collect();

                    wallet_data.resource_data
                        .fungibles
                        .insert(account_update.account.address.clone(), fungibles);

                    let non_fungibles = account_update.non_fungibles.into_values().collect();

                    wallet_data.resource_data
                        .non_fungibles
                        .insert(account_update.account.address.clone(), non_fungibles);

                    wallet_data.resource_data.accounts.insert(
                        account_update.account.address.clone(),
                        account_update.account,
                    );
                }
            }

            wallet_data.resource_data.resources = accounts_update.new_resources;
            wallet_data.resource_data.resource_icons = setup.get_icons().await;
            
            Ok(Wallet { state: Unlocked, wallet_data })
        }
    }
}