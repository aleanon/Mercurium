use std::{
    collections::HashMap,
    fmt::Display,
};

use bip39::Mnemonic;
use iced::{
    widget::image::Handle, Task,
};
use types::{
    address::ResourceAddress, collections::AccountsUpdate, crypto::{DataBaseKey, Key, PasswordError, Salt}, Account, AppError, Network
};

use crate::{
    app::{AppData, AppMessage},
    error::errorscreen::ErrorMessage,
};

use super::pages::{choose_account::{self, ChooseAccounts}, enter_seedphrase::{self, EnterSeedPhrase}, set_password::{self, SetPassword}};


#[derive(Debug)]
pub struct AccountsData {
    pub update_account_task_nr: u8,
    pub create_accounts_task_nr: u8,
    pub accounts: Vec<Vec<(Account, bool, AccountSummary)>>,
    pub page_index: usize,
    pub accounts_update: AccountsUpdate,
    pub selected_accounts: Vec<Account>,
}
// #[derive(Debug, Clone)]
// pub enum Message {
//     InputSeedWord((usize, String)),
//     PasteSeedPhrase((usize, Vec<String>)),
//     ToggleSeedPassword,
//     InputSeedPassword(String),
//     AccountsCreated(Vec<Account>),
//     InputPassword(String),
//     InputVerifyPassword(String),
//     DbAndMnemonicKeySaltReceived((DataBaseKey, Salt), (Key, Salt)),
//     ToggleAccountSelection((usize, usize)),
//     InputAccountName((usize, String)),
//     AccountsUpdated((AppData, BTreeMap<ResourceAddress, String>)),
//     IconsReceived(HashMap<ResourceAddress, Handle>),
//     NewPage(usize),
//     Complete,
//     Next,
//     Back,
// }

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Back,
    EnterSeedPhraseMessage(enter_seedphrase::Message),
    SetPasswordMessage(set_password::Message),
    ChooseAccountMessage(choose_account::Message),
}

// impl Into<AppMessage> for Message {
//     fn into(self) -> AppMessage {
//         AppMessage::Setup(super::setup::Message::RestoreFromSeedMessage(self))
//     }
// }

#[derive(Debug)]
pub enum Page {
    EnterSeedPhrase(EnterSeedPhrase),
    SetPassword(SetPassword),
    ChooseAccounts(ChooseAccounts),
    // NameAccounts(NameAccounts),
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

#[derive(Debug)]
pub struct TaskResponse<T> {
    task_id: u8,
    data: Option<T>
}

impl<T> TaskResponse<T> {
    pub fn new() -> Self {
        Self { task_id: 0, data: None }
    }

    pub fn new_response(&mut self, new_response: TaskResponse<T>) {
        if self.task_id < new_response.task_id {*self = new_response}
    }

    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    pub fn take_data(&mut self) -> Option<T> {
        self.data.take()
    }
}

#[derive(Debug)]
pub struct RestoreFromSeed {
    pub page: Page,
    key_and_salt: TaskResponse<KeyAndSalt>,
    accounts_data: TaskResponse<AccountsData>,
    icons_data: TaskResponse<IconsData>,
    
}

impl<'a> RestoreFromSeed {
    pub fn new() -> Self {
       Self {
        page: Page::EnterSeedPhrase(EnterSeedPhrase::new()),
        icons_data: TaskResponse::new(),
        accounts_data: TaskResponse::new(),
        key_and_salt: TaskResponse::new()
       }
    }

    pub fn update(
        &mut self,
        message: Message,
        appdata: &'a mut AppData,
    ) -> Result<Task<AppMessage>, AppError> {
        match message {
            Message::Next => self.next_page(appdata),
            Message::Back => self.previous_page(),
            _ => {}
        }
        Ok(Task::none())
    }

    fn next_page(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        let mut task = Task::none();
        match self.page {
            Page::EnterSeedPhrase(enter_seedphrase) => {
                let set_password_page= match SetPassword::from_page_enter_seedphrase(enter_seedphrase) {
                    Ok(page) => page,
                    Err(err) => {
                        enter_seedphrase.notification = err;
                        return Task::none();
                    }
                };
                task = self.task_create_accounts_from_seed(
                    set_password_page.seed_password.clone(), 
                    appdata.settings.network, 
                    set_password_page.mnemonic.clone()
                );
                self.page = Page::SetPassword(set_password_page);
            },
            Page::SetPassword(set_password) => task = self.goto_page_choose_account(),
            Page::ChooseAccounts(choose_accounts) => self.goto_page_name_accounts(),
            // Page::NameAccounts => task = self.finalize_setup(appdata),
            Page::Finalizing => {}
        }

        task
    }

    fn back(&mut self) {
        match self.page {
            Page::EnterSeedPhrase => {}
            Page::SetPassword => {
                self.notification = "";
                self.mnemonic = None;
                for chunk in &mut self.accounts_data.accounts {
                    chunk.clear()
                }
                self.page = Page::EnterSeedPhrase
            }
            Page::ChooseAccounts => {
                self.notification = "";
                self.key_and_salt.db_key_salt = None;
                self.key_and_salt.mnemonic_key_salt = None;
                self.page = Page::SetPassword
            }
           Page::NameAccounts => {
                self.notification = "";
                self.accounts_data.selected_accounts.clear();
                self.page = Page::ChooseAccounts
            }
            Page::Finalizing => { /*No back button at this Page*/ }
        }
    }


    fn task_derive_encryption_keys_and_salt_for_mnemonic_and_database(
        &mut self,
    ) -> Task<AppMessage> {
        let password = self.inputs.password.clone();
        let task_id = self.key_and_salt.last_task_nr + 1;
        Task::perform(
            async move {
                let db_key_salt = password.derive_new_db_encryption_key()?;
                let mnemonic_key_salt = password.derive_new_mnemonic_encryption_key()?;

                Ok::<_, PasswordError>((task_id, db_key_salt, mnemonic_key_salt))
            },
            |result| match result {
                Ok((task_id, db_key_salt, mnemonic_key_salt)) => {
                    Message::TaskResponse(TaskResponse::DbAndMnemonicKeySaltReceived {
                        task_id,
                        db_key_salt,
                        mnemonic_key_salt,
                    })
                    .into()
                }
                Err(err) => AppMessage::Error(ErrorMessage::Fatal(err.to_string())),
            },
        )
    }

    fn task_create_accounts_from_seed(
        &self,
        seed_password: Option<Password>,
        network: Network,
        mnemonic: Mnemonic,
    ) -> Task<AppMessage> {
        let task_id = self.accounts_data.task_id;
        Task::perform(
            async move {
                let password_as_str = seed_password
                    .as_ref()
                    .and_then(|password| Some(password.as_str()));

                let accounts = handles::wallet::create_multiple_accounts_from_mnemonic::<Vec<_>>(
                    &mnemonic,
                    password_as_str,
                    0,
                    0,
                    60,
                    network,
                );
                (task_id, accounts)
            },
            |(task_id, accounts)| {
                Message::TaskResponse(TaskResponse::AccountsCreated { task_id, accounts }).into()
            },
        )
    }

}
