use bip39::Mnemonic;
use types::crypto::{Password, SeedPhrase};

use super::new_wallet::NewWalletStage;

#[derive(Debug)]
pub struct NewWallet {
    pub(crate) stage: NewWalletStage,
    pub(crate) notification: &'static str,
    pub(crate) password: Password,
    pub(crate) verify_password: Password,
    pub(crate) account_name: String,
    pub(crate) mnemonic: Option<Mnemonic>,
    pub(crate) seed_phrase: SeedPhrase,
}
