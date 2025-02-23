use std::{
    collections::{BTreeMap, HashMap}, mem, sync::Arc
};

use bip39::Mnemonic;
use debug_print::debug_println;
use iced::{
    widget::{self, column, container, image::Handle, Text}, Element, Length, Task
};
use store::{DataBase, IconsDb};
use types::{
    address::ResourceAddress, collections::AccountsUpdate, crypto::{EncryptedMnemonic, HashedPassword, Key, KeySaltPair, KeyType, Password, PasswordError, Salt}, debug_info, Account, AccountSummary, AppError, AppPath, AppPathInner, Network, TaskResponse, UnsafeRef, UnsafeRefMut
};

use crate::{
    app::{AppData, AppMessage}, error::errorscreen::ErrorMessage, external_task_response, App
};

use super::{
    common::{nav_button, nav_row}, 
    pages::{choose_account::{self, ChooseAccounts}, 
    enter_seedphrase::{self, EnterSeedPhrase}, 
    name_accounts::{self, NameAccounts}, 
    set_password::{self, SetPassword}}
};


#[derive(Debug, Clone)]
pub struct AccountsData {
    pub accounts: Vec<Vec<(Account, bool, AccountSummary)>>,
    pub page_index: usize,
    pub accounts_update: AccountsUpdate,
    pub selected_accounts: Vec<Account>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Back,
    DbAndMnemonicKeySaltReceived(TaskResponse<KeysWithSalt>),
    AccountsCreated(TaskResponse<Vec<(Account, bool, AccountSummary)>>),
    AccountsUpdated(TaskResponse<AccountsUpdate>),
    IconsReceived(TaskResponse<HashMap<ResourceAddress, Handle>>),
    EnterSeedPhraseMessage(enter_seedphrase::Message),
    SetPasswordMessage(set_password::Message),
    ChooseAccountMessage(choose_account::Message),
    NameAccountsMessage(name_accounts::Message),
    Complete,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(super::setup::Message::RestoreFromSeedMessage(self))
    }
}

#[derive(Debug)]
pub enum Page {
    Placeholder,
    EnterSeedPhrase(EnterSeedPhrase),
    SetPassword(SetPassword),
    ChooseAccounts(ChooseAccounts),
    NameAccounts(NameAccounts),
    Finalizing,
}


#[derive(Debug, Clone)]
pub struct KeysWithSalt {
    pub db_key_salt: KeySaltPair<DataBase>, 
    pub mnemonic_key_salt: KeySaltPair<EncryptedMnemonic>,
}

impl KeysWithSalt {
    pub fn new(db_key_salt: KeySaltPair<DataBase>, mnemonic_key_salt: KeySaltPair<EncryptedMnemonic>) -> Self {
        Self {
           db_key_salt,
           mnemonic_key_salt
        }
    }
}


#[derive(Debug)]
pub struct RestoreFromSeed {
    pub page: Page,
    key_and_salt: TaskResponse<KeysWithSalt>,
    accounts: TaskResponse<Vec<(Account, bool, AccountSummary)>>,
    accounts_update: TaskResponse<AccountsUpdate>,
    icons_data: TaskResponse<HashMap<ResourceAddress, Handle>>,
    
}

impl<'a> RestoreFromSeed {
    pub fn new() -> Self {
        Self {
            page: Page::EnterSeedPhrase(EnterSeedPhrase::new()),
            accounts: TaskResponse::new(0, None),
            accounts_update: TaskResponse::new(0, None),
            key_and_salt: TaskResponse::new(0, None),
            icons_data: TaskResponse::new(0, None),
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        appdata: &'a mut AppData,
    ) -> Result<Task<AppMessage>, AppError> {
        let mut task = Task::none();
        match message {
            Message::Next => task = self.next_page(appdata),
            Message::Back => self.previous_page(),
            Message::AccountsCreated(task_response) => task = self.process_response_accounts_created(task_response, &appdata),
            Message::DbAndMnemonicKeySaltReceived(task_response) => self.key_and_salt.new_response(task_response), 
            Message::AccountsUpdated(task_response) => task = self.process_response_accounts_updated(task_response, &appdata),
            Message::IconsReceived(task_response) => self.icons_data.new_response(task_response),
            Message::ChooseAccountMessage(message) => {
                if let Page::ChooseAccounts(choose_account) = &mut self.page {
                    return choose_account.update(message, self.accounts.ref_mut_data())
                }
            }
            Message::EnterSeedPhraseMessage(message) => {
                if let Page::EnterSeedPhrase(enter_seedphrase) = &mut self.page {
                    return enter_seedphrase.update(message)
                }
            }
            Message::SetPasswordMessage(message) => {
                if let Page::SetPassword(set_password) = &mut self.page {
                    return set_password.update(message)
                }
            }
            Message::NameAccountsMessage(message) => {
                if let Page::NameAccounts(name_accounts) = &mut self.page {
                    return name_accounts.update(message)
                }
            }
            Message::Complete => {/*Message should just be propagated to the top level*/}
        }
        Ok(task)
    }

    fn next_page(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        let mut task = Task::none();
        let page = mem::replace(&mut self.page, Page::Placeholder);
        
        match page {
            Page::Placeholder => unreachable!("{}",debug_info!("Invalid state: Page::Placeholder")),
            Page::EnterSeedPhrase(enter_seedphrase) => task = self.from_page_enter_seedphrase_to_set_password(enter_seedphrase, appdata),
            Page::SetPassword(set_password) => task = self.from_page_set_password_to_choose_accounts(set_password),
            Page::ChooseAccounts(choose_accounts) => {
                self.page = Page::NameAccounts(NameAccounts::from_page_choose_accounts(choose_accounts, self.accounts.ref_data()));
            },
            Page::NameAccounts(name_accounts) => {
                task = self.finalize_setup(appdata, name_accounts);
                self.page = Page::Finalizing
            }
            Page::Finalizing => {}
        }
        // self.page = page;
        task
    }

    fn from_page_enter_seedphrase_to_set_password(&mut self, mut page: EnterSeedPhrase, appdata: &'a mut AppData) -> Task<AppMessage> {
        let Ok(mnemonic) = Mnemonic::from_phrase(page.seed_phrase.phrase().as_str(), bip39::Language::English) else {
            page.notification = "Invalid seed phrase";
            self.page = Page::EnterSeedPhrase(page);
            return Task::none();
        };
        let seed_password = page.seed_password.clone();
        self.page = Page::SetPassword(SetPassword::with_mnemonic_and_password(mnemonic.clone(), page.seed_password));
        Self::task_create_accounts_from_seed(seed_password, appdata.settings.network, mnemonic, self.accounts.new_task_id())
    }

    fn from_page_set_password_to_choose_accounts(&mut self, mut page: SetPassword) -> Task<AppMessage> {
        if page.password != page.verify_password {
            page.notification = "Passwords do not match";
            self.page = Page::SetPassword(page);
            return Task::none();
        } else if page.password.len() < Password::MIN_LEN {
            page.notification = "Password must be at least 16 characters long";
            self.page = Page::SetPassword(page);
            return Task::none();
        }

        let new_page = ChooseAccounts::from_page_set_password(page);
        let password = new_page.password.clone();
        self.page = Page::ChooseAccounts(new_page);

        Self::task_derive_encryption_keys_and_salt_for_mnemonic_and_database(password, self.key_and_salt.new_task_id())
    }

    fn previous_page(&mut self) {
        let page = mem::replace(&mut self.page, Page::Placeholder);

        match page {
            Page::Placeholder => unreachable!("{}", debug_info!("Invalid state: Page::Placeholder")),
            Page::EnterSeedPhrase(_) => self.page = page,
            Page::SetPassword(set_password) => {
                self.page = Page::EnterSeedPhrase(EnterSeedPhrase::from_page_set_password(set_password));
                self.accounts.discard_data();
            }
            Page::ChooseAccounts(choose_account) => {
                self.page = Page::SetPassword(SetPassword::from_page_choose_account(choose_account));
                self.key_and_salt.discard_data();
            }
           Page::NameAccounts(name_accounts) => {
                self.page = Page::ChooseAccounts(ChooseAccounts::from_page_name_accounts(name_accounts))
            }
            Page::Finalizing => unreachable!("{}", debug_info!("Back called from page: Finalizing"))
        }
    }


    fn task_derive_encryption_keys_and_salt_for_mnemonic_and_database(
        password: Password,
        task_id: u8,
    ) -> Task<AppMessage> {
        Task::perform(
            async move {
                let db_key_salt = KeySaltPair::new(password.as_str())?;
                let mnemonic_key_salt = KeySaltPair::<EncryptedMnemonic>::new(password.as_str())?;
                let key_and_salt = KeysWithSalt::new(db_key_salt, mnemonic_key_salt);

                Ok::<_, PasswordError>(TaskResponse::new(task_id, Some(key_and_salt)))
            },
            |result| match result {
                Ok(task_response) => Message::DbAndMnemonicKeySaltReceived(task_response).into(),
                Err(err) => AppMessage::Error(ErrorMessage::Fatal(err.to_string())),
            },
        )
    }

    fn task_create_accounts_from_seed(
        seed_password: Option<Password>,
        network: Network,
        mnemonic: Mnemonic,
        task_id: u8,
    ) -> Task<AppMessage> {
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

                let accounts = accounts
                    .into_iter()
                    .map(|account| (account, false, AccountSummary::NoUpdateReceived))
                    .collect::<Vec<(Account, bool, AccountSummary)>>();

                TaskResponse::new(task_id, Some(accounts))
            },
            |task_response| {
                Message::AccountsCreated(task_response).into()
            },
        )
    }

    fn process_response_accounts_created(&mut self, accounts_response: TaskResponse<Vec<(Account, bool, AccountSummary)>>, appdata: &'a AppData) -> Task<AppMessage> {
        self.accounts.new_response(accounts_response);

        let Some(accounts) = self.accounts.ref_data() else {
            return Task::none()
        };

        let accounts = accounts.iter()
            .map(|(account,_,_)| account.clone())
            .collect();

        Self::task_update_created_accounts(
            self.accounts_update.new_task_id(), 
            accounts, 
            appdata.settings.network
        )        
    }

    fn task_update_created_accounts(
        task_id: u8,
        accounts: Vec<Account>,
        network: Network,
    ) -> Task<AppMessage> {
        Task::perform(
            async move {
                let accounts_update = handles::radix_dlt::updates::update_accounts(
                    network,
                    Arc::new(HashMap::new()),
                    accounts,
                )
                .await;

                TaskResponse::new(task_id, Some(accounts_update))
            },
            |task_response| {
                Message::AccountsUpdated(task_response).into()
            },
        )
    }
    
    fn process_response_accounts_updated(&mut self, mut task_response: TaskResponse<AccountsUpdate>, appdata: &'a AppData) -> Task<AppMessage> {
        if let Some(accounts_update) = task_response.ref_mut_data() {
            accounts_update.account_updates.retain(|account_update| 
                !account_update.fungibles.is_empty() || !account_update.non_fungibles.is_empty())
        }

        self.accounts_update.new_response(task_response);

        let Some(accounts_update) = self.accounts_update.ref_data() else {
            return Task::none()
        };

        let Some(accounts) = self.accounts.ref_mut_data() else {
            return Task::none()
        };

        for (account, _, summary) in accounts {
            let account_update = accounts_update.account_updates
                .iter()
                .find(|account_update| account_update.account.address == account.address);

            match account_update {
                Some(account_update) => {
                    let account_summary = AccountSummary::Summary { 
                        nr_of_fungibles: account_update.fungibles.len(), 
                        nr_of_non_fungibles: account_update.non_fungibles.len(), 
                    };
                    *summary = account_summary;
                }
                None => {
                    let account_summary = AccountSummary::NoLedgerPresense;
                    *summary = account_summary;
                }
            }
        }

        Self::task_download_resource_icons(
            accounts_update.icon_urls.clone(), 
            appdata.settings.network, 
            self.icons_data.new_task_id()
        )
    }


    fn task_download_resource_icons(
        icon_urls: BTreeMap<ResourceAddress, String>,
        network: Network,
        task_id: u8,
    ) -> Task<AppMessage> {
        Task::perform(
            async move {
                let icons = handles::image::download::download_resize_and_store_resource_icons(
                    icon_urls, network,
                )
                .await;

                TaskResponse::new(task_id, Some(icons))
            },
            |task_response| {
                Message::IconsReceived(task_response).into()
            },
        )
    }

    fn finalize_setup(&mut self, appdata: &'a mut AppData, setup_data: NameAccounts) -> Task<AppMessage> {
        let network = appdata.settings.network;
        let accounts_update = match self.accounts_update.take_data() {
            Some(data) => data,
            None => AccountsUpdate::new(network),
        };
        let key_and_salt = self.key_and_salt.take_data();
        let icons = self.icons_data.take_data().unwrap_or(HashMap::new());

        let mut appdata = unsafe { UnsafeRefMut::new(appdata) };

        Task::perform(
            async move {
                let (db_key_salt, mnemonic_key_salt) = match key_and_salt {
                    Some(key_and_salt) => (key_and_salt.db_key_salt, key_and_salt.mnemonic_key_salt),
                    None => {
                        let db_key_salt = KeySaltPair::<DataBase>::new(setup_data.password.as_str())
                            .map_err(|err| AppError::Fatal(err.to_string()))?;

                        let mnemonic_key_salt = KeySaltPair::<EncryptedMnemonic>::new(setup_data.password.as_str())
                            .map_err(|err| AppError::Fatal(err.to_string()))?;

                        (db_key_salt, mnemonic_key_salt)
                    }
                };

                let password_hash = setup_data.password.derive_db_encryption_key_hash_from_salt(db_key_salt.salt());

                let seed_pw_as_str = setup_data
                    .seed_password
                    .as_ref()
                    .and_then(|pw| Some(pw.as_str()));

                AppPath::get().create_directories_if_not_exists()
                    .map_err(|err| AppError::Fatal(err.to_string()))?;

                let db_key = db_key_salt.key().clone();

                handles::wallet::create_new_wallet_with_accounts(
                    &setup_data.mnemonic,
                    seed_pw_as_str,
                    db_key_salt,
                    mnemonic_key_salt,
                    password_hash,
                    &setup_data.accounts,
                    network,
                )
                .await?;

                IconsDb::load(network, db_key).await
                    .map_err(|err| AppError::Fatal(err.to_string()))?;

                for account in setup_data.accounts {
                    for account_update in accounts_update.account_updates.clone() {
                        if account_update.account.address != account.address {continue};
                        
                        let fungibles = account_update.fungibles.into_values().collect();
                        appdata
                            .fungibles
                            .insert(account_update.account.address.clone(), fungibles);

                        let non_fungibles = account_update.non_fungibles.into_values().collect();
                        appdata
                            .non_fungibles
                            .insert(account_update.account.address.clone(), non_fungibles);

                        appdata.accounts.insert(
                            account_update.account.address.clone(),
                            account_update.account,
                        );
                    }
                }

                appdata.resources = accounts_update.new_resources;
                appdata.resource_icons = icons;

                Ok::<_, AppError>(())
            },
            |result| match result {
                Ok(_) => AppMessage::TaskResponse(external_task_response::Message::WalletCreated),
                Err(err) => {
                    debug_println!("{}", err.to_string());
                    AppMessage::Error(ErrorMessage::Fatal(err.to_string())).into()
                },
            },
        )
    }

}

impl<'a> RestoreFromSeed {
    pub fn view(&self, app: &'a App) -> Element<'_, AppMessage> {
        let page: Element<'_, AppMessage> = match &self.page {
            Page::Placeholder => unreachable!("{}", debug_info!("Invalid state: Page::Placeholder")), 
            Page::EnterSeedPhrase(enter_seedphrase) => enter_seedphrase.view(),
            Page::SetPassword(set_password) => set_password.view(),
            Page::ChooseAccounts(choose_accounts) => choose_accounts.view(self.accounts.ref_data()),
            Page::NameAccounts(name_accounts) => name_accounts.view(),
            Page::Finalizing => return Text::new("Finalizing...").into(),
        };

        let page_container = container(page)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        let nav = nav_row(
            nav_button("Back").on_press(Message::Back.into()),
            nav_button("Next").on_press(Message::Next.into()),
        );

        let content = column![page_container, nav];

        widget::container(content)
            .center_x(660)
            .center_y(700)
            .into()
    }
}