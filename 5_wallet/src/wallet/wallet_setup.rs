pub mod task_manager;
pub mod task_runner;
pub mod setup;
pub mod setup_error;


use std::sync::Arc;

use setup::Setup;
use setup_error::SetupError;
use task_manager::TaskManager;
use types::{crypto::{bip39::{Language, Mnemonic}, Password, Phrase}, Account};


use super::Wallet;


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

    
}