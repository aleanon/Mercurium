use bip39::Mnemonic;
use types::{
    address::AccountAddress,
    crypto::{Password, SeedPhrase},
    Account,
};

use super::new_wallet::NewWalletStage;

#[derive(Debug)]
pub enum Stage {
    EnterSeedPhrase,
    EnterPassword,
    ChooseAccounts,
    NameAccounts,
    Finalizing,
}

#[derive(Debug)]
pub struct RestoreFromSeed {
    pub stage: Stage,
    pub notification: &'static str,
    pub seed_phrase: SeedPhrase,
    pub seed_password: Option<Password>,
    pub mnemonic: Option<Mnemonic>,
    pub password: Password,
    pub verify_password: Password,
    pub accounts: Vec<Account>,
}
