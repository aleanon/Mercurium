use std::{collections::HashMap, fmt::Display};

use bip39::Mnemonic;
use iced::widget::image::Handle;
use scrypto::crypto::Ed25519PrivateKey;
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
pub struct Inputs {
    pub seed_phrase: SeedPhrase,
    pub seed_password: Option<Password>,
    pub password: Password,
    pub verify_password: Password,
}

impl Inputs {
    pub fn new() -> Self {
        Self {
            seed_phrase: SeedPhrase::new(),
            seed_password: None,
            password: Password::new(),
            verify_password: Password::new(),
        }
    }
}

#[derive(Debug)]
pub struct KeyAndSalt {
    pub last_task_nr: u8,
    pub db_key_salt: Option<(DataBaseKey, Salt)>,
    pub mnemonic_key_salt: Option<(Key, Salt)>,
}

impl KeyAndSalt {
    pub fn new() -> Self {
        Self {
            last_task_nr: 0,
            db_key_salt: None,
            mnemonic_key_salt: None,
        }
    }
}

#[derive(Debug)]
pub struct AccountsData {
    pub update_account_task_nr: u8,
    pub create_accounts_task_nr: u8,
    pub accounts: Vec<Vec<(Account, bool, AccountSummary)>>,
    pub page_index: usize,
    pub accounts_update: AccountsUpdate,
    pub selected_accounts: Vec<Account>,
}

impl AccountsData {
    pub fn new(network: Network) -> Self {
        Self {
            update_account_task_nr: 0,
            create_accounts_task_nr: 0,
            accounts: Vec::new(),
            page_index: 0,
            accounts_update: AccountsUpdate::new(network),
            selected_accounts: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct IconsData {
    pub task_nr: u8,
    pub icons: HashMap<ResourceAddress, Handle>,
}

impl IconsData {
    pub fn new() -> Self {
        Self {
            task_nr: 0,
            icons: HashMap::new(),
        }
    }
}

// #[derive(Debug)]
// pub struct RestoreFromSeed {
//     pub stage: Stage,
//     pub notification: &'static str,
//     pub seed_phrase: SeedPhrase,
//     pub seed_password: Option<Password>,
//     pub mnemonic: Option<Mnemonic>,
//     pub password: Password,
//     pub verify_password: Password,
//     pub db_key_salt: Option<(DataBaseKey, Salt)>,
//     pub mnemonic_key_salt: Option<(Key, Salt)>,
//     pub accounts: Vec<Vec<(Account, bool, AccountSummary)>>,
//     pub page_index: usize,
//     pub accounts_update: AccountsUpdate,
//     pub icons: HashMap<ResourceAddress, Handle>,
//     pub selected_accounts: Vec<Account>,
// }

#[derive(Debug)]
pub struct RestoreFromSeed {
    pub stage: Stage,
    pub notification: &'static str,
    pub inputs: Inputs,
    pub mnemonic: Option<Mnemonic>,
    pub key_and_salt: KeyAndSalt,
    pub accounts_data: AccountsData,
    pub icons_data: IconsData,
}

impl<'a> RestoreFromSeed {
    // pub fn new(network: Network) -> Self {
    //     Self {
    //         stage: Stage::EnterSeedPhrase,
    //         notification: "",
    //         seed_phrase: SeedPhrase::new(),
    //         seed_password: None,
    //         mnemonic: None,
    //         password: Password::new(),
    //         verify_password: Password::new(),
    //         db_key_salt: None,
    //         mnemonic_key_salt: None,
    //         accounts: vec![Vec::with_capacity(20); 20],
    //         page_index: 0,
    //         accounts_update: AccountsUpdate::new(network),
    //         icons: HashMap::new(),
    //         selected_accounts: Vec::new(),
    //     }
    // }

    pub struct TaskResponse<T> {
        task_id: u8,
        data: Option<T>
    }

    impl<T> TaskResponse<T> {
        pub fn new() -> Self {
            Self { 0, None }
        }
    }

    pub fn new(network: Network) -> Self {
        Self {
            stage: Stage::EnterSeedPhrase,
            notification: "",
            inputs: Inputs::new(),
            mnemonic: None,
            key_and_salt: KeyAndSalt::new(),
            accounts_data: AccountsData::new(network),
            icons_data: IconsData::new(),
        }
    }
}
