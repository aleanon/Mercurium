use std::{collections::HashMap, fmt::Display};

use bip39::Mnemonic;
use iced::widget::image::Handle;
use types::{
    address::ResourceAddress,
    collections::AccountsUpdate,
    crypto::{DataBaseKey, Key, Password, Salt, SeedPhrase},
    Account, Network,
};

#[derive(Debug)]
pub enum Stage {
    EnterSeedPhrase,
    EnterPassword,
    ChooseAccounts,
    NameAccounts,
    Finalizing,
}

#[derive(Debug, Clone)]
pub enum AccountSummary {
    NoUpdateReceived,
    NoLedgerPresense,
    Summary {
        nr_of_fungibles: usize,
        nr_of_non_fungibles: usize,
    },
}

impl Display for AccountSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoUpdateReceived => write!(f, "No update"),
            Self::NoLedgerPresense => write!(f, "None"),
            Self::Summary {
                nr_of_fungibles,
                nr_of_non_fungibles,
            } => write!(
                f,
                "{} Fungibles, {} NFTs",
                nr_of_fungibles, nr_of_non_fungibles
            ),
        }
    }
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
    pub db_key_salt: Option<(DataBaseKey, Salt)>,
    pub mnemonic_key_salt: Option<(Key, Salt)>,
    pub accounts: Vec<Vec<(Account, bool, AccountSummary)>>,
    pub page_index: usize,
    pub accounts_update: AccountsUpdate,
    pub icons: HashMap<ResourceAddress, Handle>,
    pub selected_accounts: Vec<Account>,
}

impl<'a> RestoreFromSeed {
    pub fn new(network: Network) -> Self {
        Self {
            stage: Stage::EnterSeedPhrase,
            notification: "",
            seed_phrase: SeedPhrase::new(),
            seed_password: None,
            mnemonic: None,
            password: Password::new(),
            verify_password: Password::new(),
            db_key_salt: None,
            mnemonic_key_salt: None,
            accounts: vec![Vec::with_capacity(20); 20],
            page_index: 0,
            accounts_update: AccountsUpdate::new(network),
            icons: HashMap::new(),
            selected_accounts: Vec::new(),
        }
    }
}
