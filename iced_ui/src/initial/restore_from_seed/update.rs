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
    crypto::{DataBaseKey, Key, Password, PasswordError, Salt},
    Account, AppError, AppSettings, MutUr, Ur,
};
use zeroize::Zeroize;

use crate::{
    app::{AppData, AppMessage},
    error::errorscreen::ErrorMessage,
};

#[derive(Debug, Clone)]
pub enum Message {
    InputSeedWord((usize, String)),
    PasteSeedPhrase((usize, Vec<String>)),
    ToggleSeedPassword,
    InputSeedPassword(String),
    AccountsCreated(Vec<Account>),
    InputPassword(String),
    InputVerifyPassword(String),
    DbAndMnemonicKeySaltReceived((DataBaseKey, Salt), (Key, Salt)),
    ToggleAccountSelection((usize, usize)),
    InputAccountName((usize, String)),
    AccountsUpdated((AppData, BTreeMap<ResourceAddress, String>)),
    IconsReceived(HashMap<ResourceAddress, Handle>),
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
        match message {
            Message::InputSeedWord((word_index, mut word)) => {
                self.seed_phrase.update_word(word_index, word.as_str());
                word.zeroize();
            }
            Message::PasteSeedPhrase((mut index, words)) => {
                for mut word in words {
                    self.seed_phrase.update_word(index, &word);
                    word.zeroize();
                    index += 1;
                }

                if let Ok(mnemonic) = Mnemonic::from_phrase(
                    self.seed_phrase.phrase().as_str(),
                    bip39::Language::English,
                ) {
                    self.mnemonic = Some(mnemonic);
                }
            }
            Message::ToggleSeedPassword => {
                if self.seed_password.is_none() {
                    self.seed_password = Some(Password::new())
                } else {
                    self.seed_password = None;
                }
            }
            Message::InputSeedPassword(input) => self.update_input_seed_password_field(input),
            Message::AccountsCreated(accounts) => {
                return self.save_created_accounts(accounts, appdata)
            }
            Message::InputPassword(input) => self.update_input_password_field(input),
            Message::InputVerifyPassword(mut input) => {
                self.verify_password.clear();
                self.verify_password.push_str(input.as_str());
                input.zeroize();
                self.notification = "";
            }
            Message::DbAndMnemonicKeySaltReceived(db_key_salt, mnemonic_key_salt) => {
                match self.stage {
                    Stage::ChooseAccounts | Stage::NameAccounts => {
                        self.db_key_salt = Some(db_key_salt);
                        self.mnemonic_key_salt = Some(mnemonic_key_salt);
                    }
                    _ => {}
                }
            }
            Message::ToggleAccountSelection((chunk_index, account_index)) => {
                if let Some(chunk) = self.accounts.get_mut(chunk_index) {
                    if let Some((_, is_selected, _)) = chunk.get_mut(account_index) {
                        *is_selected = !*is_selected
                    }
                }
            }
            Message::AccountsUpdated((new_appdata, icon_urls)) => {
                appdata.accounts = new_appdata.accounts;
                appdata.fungibles = new_appdata.fungibles;
                appdata.non_fungibles = new_appdata.non_fungibles;
                appdata.resources = new_appdata.resources;

                for chunk in self.accounts.iter_mut() {
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

                let network = appdata.settings.network;
                return Ok(Task::perform(
                    async move {
                        handles::image::download::download_resize_and_store_resource_icons(
                            icon_urls, network,
                        )
                        .await
                    },
                    |icons| Message::IconsReceived(icons).into(),
                ));
            }
            Message::IconsReceived(icons) => appdata.resource_icons = icons,
            Message::InputAccountName((index, account_name)) => {
                if let Some(account) = self.selected_accounts.get_mut(index) {
                    account.name = account_name
                }
            }
            Message::NewPage(page_index) => self.page_index = page_index,
            Message::Complete => {}
            Message::Next => return Ok(self.next(appdata)),
            Message::Back => self.back(),
        }

        Ok(Task::none())
    }

    fn save_created_accounts(
        &mut self,
        accounts: Vec<Account>,
        appdata: &mut AppData,
    ) -> Result<Task<AppMessage>, AppError> {
        match self.stage {
            Stage::EnterSeedPhrase => {
                // If the used has gone back we just want to drop the created accounts
                return Ok(Task::none());
            }
            _ => {
                self.accounts = accounts
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

                let network = appdata.settings.network;
                Ok(Task::perform(
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
                        let mut non_fungible_tokens: HashMap<
                            AccountAddress,
                            BTreeSet<NonFungibleAsset>,
                        > = HashMap::new();

                        for account_update in accounts_update.account_updates {
                            let fungibles = account_update.fungibles.into_values().collect();

                            fungible_tokens
                                .insert(account_update.account.address.clone(), fungibles);

                            let non_fungibles =
                                account_update.non_fungibles.into_values().collect();

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

                        (appdata, accounts_update.icon_urls)
                    },
                    |accounts_update| Message::AccountsUpdated(accounts_update).into(),
                ))
            }
        }
    }

    fn update_input_seed_password_field(&mut self, mut input: String) {
        self.seed_password
            .as_mut()
            .and_then(|password| Some(password.replace(input.as_str())));

        input.zeroize();
    }

    fn update_input_password_field(&mut self, mut input: String) {
        self.password.clear();
        self.password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }

    fn next(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        match self.stage {
            Stage::EnterSeedPhrase => {
                let mnemonic = Mnemonic::from_phrase(
                    self.seed_phrase.phrase().as_str(),
                    bip39::Language::English,
                );
                let Ok(mnemonic) = mnemonic else {
                    self.notification = "Invalid Mnemonic seed phrase, please try again";
                    return Task::none();
                };

                self.mnemonic = Some(mnemonic.clone());
                self.stage = Stage::EnterPassword;

                let password = self.seed_password.clone();
                let network = appdata.settings.network;

                return Task::perform(
                    async move {
                        let password_as_str = password
                            .as_ref()
                            .and_then(|password| Some(password.as_str()));

                        handles::wallet::create_multiple_accounts_from_mnemonic::<Vec<_>>(
                            &mnemonic,
                            password_as_str,
                            0,
                            0,
                            60,
                            network,
                        )
                    },
                    |accounts| Message::AccountsCreated(accounts).into(),
                );
            }
            Stage::EnterPassword => {
                if self.password != self.verify_password {
                    self.notification = "Passwords do not match";
                    return Task::none();
                } else if self.password.len() < Password::MIN_LEN {
                    self.notification = "Password must be at least 16 characters long";
                    return Task::none();
                } else {
                    self.notification = "";
                    self.stage = Stage::ChooseAccounts;

                    let password = self.password.clone();
                    return Task::perform(
                        async move {
                            let db_key_salt = password.derive_new_db_encryption_key()?;
                            let mnemonic_key_salt =
                                password.derive_new_mnemonic_encryption_key()?;

                            Ok::<_, PasswordError>((db_key_salt, mnemonic_key_salt))
                        },
                        |db_and_mnemonic_key_salt| match db_and_mnemonic_key_salt {
                            Ok(db_and_mnemonic_key_salt) => Message::DbAndMnemonicKeySaltReceived(
                                db_and_mnemonic_key_salt.0,
                                db_and_mnemonic_key_salt.1,
                            )
                            .into(),
                            Err(err) => AppMessage::Error(ErrorMessage::Fatal(err.to_string())),
                        },
                    );
                }
            }
            Stage::ChooseAccounts => {
                self.selected_accounts = self
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
                for chunk in &mut self.accounts {
                    chunk.clear()
                }
                self.stage = Stage::EnterSeedPhrase
            }
            Stage::ChooseAccounts => {
                self.notification = "";
                self.db_key_salt = None;
                self.mnemonic_key_salt = None;
                self.stage = Stage::EnterPassword
            }
            Stage::NameAccounts => {
                self.notification = "";
                self.selected_accounts.clear();
                self.stage = Stage::ChooseAccounts
            }
            Stage::Finalizing => { /*No back button at this stage*/ }
        }
    }

    fn finalize_setup(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        self.stage = Stage::Finalizing;
        let network = appdata.settings.network;
        let setup_data = unsafe { Ur::new(self) };
        let accounts_update =
            std::mem::replace(&mut self.accounts_update, AccountsUpdate::new(network));
        let icons = std::mem::take(&mut self.icons);
        let mut appdata = unsafe { MutUr::new(appdata) };

        let create_wallet = Task::perform(
            async move {
                let (db_key, db_salt) = match &setup_data.db_key_salt {
                    Some(key_and_salt) => key_and_salt.clone(),
                    None => setup_data
                        .password
                        .derive_new_db_encryption_key()
                        .map_err(|err| AppError::Fatal(err.to_string()))?,
                };

                let (mnemonic_key, mnemonic_salt) = match &setup_data.mnemonic_key_salt {
                    Some(key_and_salt) => key_and_salt.clone(),
                    None => setup_data
                        .password
                        .derive_new_mnemonic_encryption_key()
                        .map_err(|err| AppError::Fatal(err.to_string()))?,
                };

                let mnemonic = match &setup_data.mnemonic {
                    Some(mnemonic) => mnemonic,
                    None => &Mnemonic::from_phrase(
                        setup_data.seed_phrase.phrase().as_str(),
                        bip39::Language::English,
                    )
                    .map_err(|err| AppError::Fatal(err.to_string()))?,
                };

                let seed_pw_as_str = setup_data
                    .seed_password
                    .as_ref()
                    .and_then(|pw| Some(pw.as_str()));

                handles::wallet::create_new_wallet_with_accounts(
                    mnemonic,
                    seed_pw_as_str,
                    (db_key, db_salt),
                    (mnemonic_key, mnemonic_salt),
                    &setup_data.selected_accounts,
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
