use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use super::{super::setup, Stage};
use super::{AccountSummary, RestoreFromSeed};
use bip39::Mnemonic;
use iced::{widget::image::Handle, Task};
use types::{
    address::{AccountAddress, ResourceAddress},
    assets::{FungibleAsset, NonFungibleAsset},
    collections::AccountsUpdate,
    crypto::{DataBaseKey, Key, Salt},
    Account, AppError, AppSettings, MutUr, Network, Ur,
};

use crate::{
    app::{AppData, AppMessage},
    error::errorscreen::ErrorMessage,
};

#[derive(Debug, Clone)]
pub enum TaskResponse {
    DbAndMnemonicKeySaltReceived {
        task_id: u8,
        db_key_salt: (DataBaseKey, Salt),
        mnemonic_key_salt: (Key, Salt),
    },
    AccountsCreated {
        task_id: u8,
        accounts: Vec<Account>,
    },
    AccountsUpdated {
        task_id: u8,
        new_appdata: AppData,
        icon_urls: BTreeMap<ResourceAddress, String>,
    },
    IconsReceived {
        task_id: u8,
        icons: HashMap<ResourceAddress, Handle>,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    TaskResponse(TaskResponse),
    InputSeedWord((usize, String)),
    PasteSeedPhrase((usize, Vec<String>)),
    ToggleSeedPassword,
    InputSeedPassword(String),
    InputPassword(String),
    InputVerifyPassword(String),
    ToggleAccountSelection((usize, usize)),
    InputAccountName((usize, String)),
    NewPage(usize),
    Complete,
    Next,
    Back,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(setup::Message::RestoreFromSeedMessage(self))
    }
}

impl<'a> RestoreFromSeed {
    pub fn update(
        &mut self,
        message: Message,
        appdata: &'a mut AppData,
    ) -> Result<Task<AppMessage>, AppError> {
        let mut task = Task::none();
        match message {
            Message::TaskResponse(taskresponse) => {
                task = self.handle_task_response(taskresponse, appdata)
            }
            Message::InputSeedWord((word_index, word)) => {
                self.update_single_word_in_seed_phrase(word_index, word)
            }
            Message::PasteSeedPhrase((index, words)) => {
                self.update_multiple_words_in_seed_phrase(index, words)
            }
            Message::ToggleSeedPassword => self.toggle_use_of_seed_password(),
            Message::InputSeedPassword(input) => self.update_seed_password_field(input),
            Message::InputPassword(input) => self.update_password_field(input),
            Message::InputVerifyPassword(input) => self.update_verify_password_field(input),
            Message::ToggleAccountSelection((chunk_index, account_index)) => {
                self.update_account_selected(chunk_index, account_index)
            }
            Message::InputAccountName((index, account_name)) => {
                self.update_account_name(index, account_name)
            }
            Message::NewPage(page_index) => self.accounts_data.page_index = page_index,
            Message::Next => task = self.next(appdata),
            Message::Back => self.back(),
            Message::Complete => {}
        }

        Ok(task)
    }

    fn handle_task_response(
        &mut self,
        taskresponse: TaskResponse,
        appdata: &mut AppData,
    ) -> Task<AppMessage> {
        let mut task = Task::none();
        match taskresponse {
            TaskResponse::AccountsCreated { task_id, accounts } => {
                self.save_created_accounts(task_id, &accounts);
                task = self.task_update_created_accounts(
                    self.accounts_data.update_account_task_nr + 1,
                    accounts,
                    appdata.settings.network,
                );
            }
            TaskResponse::DbAndMnemonicKeySaltReceived {
                task_id,
                db_key_salt,
                mnemonic_key_salt,
            } => self.update_database_key_and_mnemonic_key_fields(
                task_id,
                db_key_salt,
                mnemonic_key_salt,
            ),
            TaskResponse::AccountsUpdated {
                task_id,
                new_appdata,
                icon_urls,
            } => {
                self.save_updated_account_data(task_id, appdata, new_appdata);
                task = self.task_download_resource_icons(
                    icon_urls,
                    appdata.settings.network,
                    self.icons_data.task_nr + 1,
                )
            }
            TaskResponse::IconsReceived { task_id, icons } => {
                if task_id > self.icons_data.task_nr {
                    self.icons_data.task_nr = task_id;
                    self.icons_data.icons = icons;
                }
            }
        }
        return task;
    }

    fn save_created_accounts(&mut self, task_id: u8, accounts: &Vec<Account>) {
        match self.stage {
            Stage::EnterSeedPhrase => {
                // If the used has gone back we just want to drop the created accounts
            }
            _ => {
                if task_id > self.accounts_data.create_accounts_task_nr {
                    self.accounts_data.accounts = accounts
                        .chunks(20)
                        .map(|chunk| {
                            chunk
                                .iter()
                                .map(|account| {
                                    (account.clone(), false, AccountSummary::NoUpdateReceived)
                                })
                                .collect()
                        })
                        .collect();

                    self.accounts_data.create_accounts_task_nr = task_id;
                }
            }
        }
    }

    fn update_account_name(&mut self, index: usize, account_name: String) {
        if let Some(account) = self.accounts_data.selected_accounts.get_mut(index) {
            account.name = account_name
        }
    }

    fn save_updated_account_data(
        &mut self,
        task_id: u8,
        appdata: &mut AppData,
        new_appdata: AppData,
    ) {
        if task_id > self.accounts_data.update_account_task_nr {
            self.accounts_data.update_account_task_nr = task_id;

            appdata.accounts = new_appdata.accounts;
            appdata.fungibles = new_appdata.fungibles;
            appdata.non_fungibles = new_appdata.non_fungibles;
            appdata.resources = new_appdata.resources;

            for chunk in self.accounts_data.accounts.iter_mut() {
                for (account, _, account_summary) in chunk {
                    let mut account_sum = AccountSummary::NoLedgerPresense;
                    if let Some(fungibles) = appdata.fungibles.get(&account.address) {
                        account_sum = AccountSummary::Summary {
                            nr_of_fungibles: fungibles.len(),
                            nr_of_non_fungibles: 0,
                        };
                    }
                    if let Some(non_fungibles) = appdata.non_fungibles.get(&account.address) {
                        match &mut account_sum {
                            AccountSummary::Summary {
                                nr_of_non_fungibles,
                                ..
                            } => *nr_of_non_fungibles = non_fungibles.len(),
                            AccountSummary::NoLedgerPresense => {
                                account_sum = AccountSummary::Summary {
                                    nr_of_fungibles: 0,
                                    nr_of_non_fungibles: non_fungibles.len(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    *account_summary = account_sum;
                }
            }
        }
    }

    fn task_download_resource_icons(
        &mut self,
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

                (task_id, icons)
            },
            |(task_id, icons)| {
                Message::TaskResponse(TaskResponse::IconsReceived { task_id, icons }).into()
            },
        )
    }

    fn update_account_selected(&mut self, chunk_index: usize, account_index: usize) {
        if let Some(chunk) = self.accounts_data.accounts.get_mut(chunk_index) {
            if let Some((_, is_selected, _)) = chunk.get_mut(account_index) {
                *is_selected = !*is_selected
            }
        }
    }

    fn update_database_key_and_mnemonic_key_fields(
        &mut self,
        task_id: u8,
        db_key_salt: (DataBaseKey, Salt),
        mnemonic_key_salt: (Key, Salt),
    ) {
        match self.stage {
            Stage::ChooseAccounts | Stage::NameAccounts => {
                if task_id > self.key_and_salt.last_task_nr {
                    self.key_and_salt.last_task_nr = task_id;
                    self.key_and_salt.db_key_salt = Some(db_key_salt);
                    self.key_and_salt.mnemonic_key_salt = Some(mnemonic_key_salt);
                }
            }
            _ => { /* */ }
        }
    }

    // fn update_verify_password_field(&mut self, mut input: String) {
    //     self.inputs.verify_password.clear();
    //     self.inputs.verify_password.push_str(input.as_str());
    //     input.zeroize();
    //     self.notification = "";
    // }

    // fn update_single_word_in_seed_phrase(&mut self, word_index: usize, mut word: String) {
    //     self.inputs
    //         .seed_phrase
    //         .update_word(word_index, word.as_str());
    //     word.zeroize();
    //     self.notification = "";
    // }

    fn task_update_created_accounts(
        &mut self,
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

                let mut accounts: HashMap<AccountAddress, Account> = HashMap::new();
                let mut fungible_tokens: HashMap<AccountAddress, BTreeSet<FungibleAsset>> =
                    HashMap::new();
                let mut non_fungible_tokens: HashMap<AccountAddress, BTreeSet<NonFungibleAsset>> =
                    HashMap::new();

                for account_update in accounts_update.account_updates {
                    let fungibles = account_update.fungibles.into_values().collect();

                    fungible_tokens.insert(account_update.account.address.clone(), fungibles);

                    let non_fungibles = account_update.non_fungibles.into_values().collect();

                    non_fungible_tokens
                        .insert(account_update.account.address.clone(), non_fungibles);

                    accounts.insert(
                        account_update.account.address.clone(),
                        account_update.account,
                    );
                }

                let appdata = AppData {
                    accounts,
                    fungibles: fungible_tokens,
                    non_fungibles: non_fungible_tokens,
                    resources: accounts_update.new_resources,
                    resource_icons: HashMap::new(),
                    settings: AppSettings::new(),
                };

                (task_id, appdata, accounts_update.icon_urls)
            },
            |(task_id, new_appdata, icon_urls)| {
                Message::TaskResponse(TaskResponse::AccountsUpdated {
                    task_id,
                    new_appdata,
                    icon_urls,
                })
                .into()
            },
        )
    }

    // fn update_multiple_words_in_seed_phrase(&mut self, mut index: usize, words: Vec<String>) {
    //     for mut word in words {
    //         self.inputs.seed_phrase.update_word(index, &word);
    //         word.zeroize();
    //         index += 1;
    //     }
    // }

    // fn toggle_use_of_seed_password(&mut self) {
    //     if self.inputs.seed_password.is_none() {
    //         self.inputs.seed_password = Some(Password::new())
    //     } else {
    //         self.inputs.seed_password = None;
    //     }
    // }

    // fn update_seed_password_field(&mut self, mut input: String) {
    //     self.inputs
    //         .seed_password
    //         .as_mut()
    //         .and_then(|password| Some(password.replace(input.as_str())));

    //     input.zeroize();
    // }

    // fn update_password_field(&mut self, mut input: String) {
    //     self.inputs.password.clear();
    //     self.inputs.password.push_str(input.as_str());
    //     input.zeroize();
    //     self.notification = "";
    // }

    fn next(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        match self.stage {
            Stage::EnterSeedPhrase => return self.from_enter_seed_to_enter_password(appdata),
            Stage::EnterPassword => return self.from_enter_password_to_choose_account(),
            Stage::ChooseAccounts => {
                self.accounts_data.selected_accounts = self
                    .accounts_data
                    .accounts
                    .iter()
                    .flatten()
                    .filter_map(|(account, selected, _)| selected.then_some(account.clone()))
                    .collect();

                self.stage = Stage::NameAccounts;
            }
            Stage::NameAccounts => return self.finalize_setup(appdata),
            Stage::Finalizing => {}
        }

        Task::none()
    }

    fn back(&mut self) {
        match self.stage {
            Stage::EnterSeedPhrase => {}
            Stage::EnterPassword => {
                self.notification = "";
                self.mnemonic = None;
                for chunk in &mut self.accounts_data.accounts {
                    chunk.clear()
                }
                self.stage = Stage::EnterSeedPhrase
            }
            Stage::ChooseAccounts => {
                self.notification = "";
                self.key_and_salt.db_key_salt = None;
                self.key_and_salt.mnemonic_key_salt = None;
                self.stage = Stage::EnterPassword
            }
            Stage::NameAccounts => {
                self.notification = "";
                self.accounts_data.selected_accounts.clear();
                self.stage = Stage::ChooseAccounts
            }
            Stage::Finalizing => { /*No back button at this stage*/ }
        }
    }

    fn finalize_setup(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        self.stage = Stage::Finalizing;
        let network = appdata.settings.network;
        let setup_data = unsafe { Ur::new(self) };
        let accounts_update = std::mem::replace(
            &mut self.accounts_data.accounts_update,
            AccountsUpdate::new(network),
        );
        let icons = std::mem::take(&mut self.icons_data.icons);
        let mut appdata = unsafe { MutUr::new(appdata) };

        let create_wallet = Task::perform(
            async move {
                let (db_key, db_salt) = match &setup_data.key_and_salt.db_key_salt {
                    Some(key_and_salt) => key_and_salt.clone(),
                    None => setup_data
                        .inputs
                        .password
                        .derive_new_db_encryption_key()
                        .map_err(|err| AppError::Fatal(err.to_string()))?,
                };

                let (mnemonic_key, mnemonic_salt) = match &setup_data.key_and_salt.mnemonic_key_salt
                {
                    Some(key_and_salt) => key_and_salt.clone(),
                    None => setup_data
                        .inputs
                        .password
                        .derive_new_mnemonic_encryption_key()
                        .map_err(|err| AppError::Fatal(err.to_string()))?,
                };

                let mnemonic = match &setup_data.mnemonic {
                    Some(mnemonic) => mnemonic,
                    None => &Mnemonic::from_phrase(
                        setup_data.inputs.seed_phrase.phrase().as_str(),
                        bip39::Language::English,
                    )
                    .map_err(|err| AppError::Fatal(err.to_string()))?,
                };

                let seed_pw_as_str = setup_data
                    .inputs
                    .seed_password
                    .as_ref()
                    .and_then(|pw| Some(pw.as_str()));

                handles::wallet::create_new_wallet_with_accounts(
                    mnemonic,
                    seed_pw_as_str,
                    (db_key, db_salt),
                    (mnemonic_key, mnemonic_salt),
                    &setup_data.accounts_data.selected_accounts,
                    network,
                )
                .await?;

                for account_update in accounts_update.account_updates {
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

                appdata.resources = accounts_update.new_resources;
                appdata.resource_icons = icons;

                Ok::<_, AppError>(())
            },
            |result| match result {
                Ok(_) => Message::Complete.into(),
                Err(err) => AppMessage::Error(ErrorMessage::Fatal(err.to_string())).into(),
            },
        );

        let move_accounts_and_resources_to_appdata =
            Task::perform(async move {}, |_| Message::Complete.into());

        Task::batch([create_wallet, move_accounts_and_resources_to_appdata])
    }
}
