use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use bip39::Mnemonic;
use iced::{
    widget::{column, image::Handle},
    Element, Task,
};
use store::AsyncDb;
use types::{
    address::ResourceAddress,
    collections::AccountsUpdate,
    crypto::{DataBaseKey, Key, Password, PasswordError, Salt, SeedPhrase},
    Account, AppError, MutUr, Network, Ur,
};
use zeroize::Zeroize;

use crate::{
    app::{AppData, AppMessage},
    error::errorscreen::ErrorMessage,
    App,
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
    AccountsUpdated(AccountsUpdate),
    IconsReceived(HashMap<ResourceAddress, Handle>),
    Complete,
    Next,
    Back,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(super::setup::Message::RestoreFromSeedMessage(self))
    }
}

#[derive(Debug)]
pub enum Stage {
    EnterSeedPhrase,
    EnterPassword,
    ChooseAccounts,
    NameAccounts,
    Finalizing,
}

#[derive(Debug, Clone)]
pub struct AccountSummary {
    nr_of_fungibles: usize,
    nr_of_non_fungibles: usize,
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
            accounts_update: AccountsUpdate::new(network),
            icons: HashMap::new(),
            selected_accounts: Vec::new(),
        }
    }

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
            Message::InputSeedPassword(mut input) => {
                self.seed_password
                    .as_mut()
                    .and_then(|password| Some(password.replace_str(input.as_str())));

                input.zeroize();
            }
            Message::AccountsCreated(accounts) => {
                match self.stage {
                    Stage::EnterSeedPhrase => { /*If the user has gone back we want to drop this value*/
                    }
                    _ => {
                        let network = appdata.settings.network;
                        return Ok(Task::perform(
                            async move {
                                handles::radix_dlt::updates::update_accounts(
                                    network,
                                    Arc::new(HashMap::new()),
                                    accounts,
                                )
                                .await
                            },
                            |accounts_update| Message::AccountsUpdated(accounts_update).into(),
                        ));
                    }
                }
            }
            Message::InputPassword(mut input) => {
                self.password.replace_str(input.as_str());
                input.zeroize()
            }
            Message::InputVerifyPassword(mut input) => {
                self.verify_password.replace_str(input.as_str());
                input.zeroize()
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
            Message::AccountsUpdated(accounts_update) => {
                self.accounts_update = accounts_update;

                let icons_urls = self.accounts_update.icon_urls;
                let network = appdata.settings.network;

                return Ok(Task::perform(
                    async move {
                        handles::image::download::download_resize_and_store_resource_icons(
                            icons_urls, network,
                        )
                        .await
                    },
                    |icons| Message::IconsReceived(icons).into(),
                ));
            }
            Message::IconsReceived(icons) => self.icons = icons,
            Message::InputAccountName((index, account_name)) => {
                if let Some(account) = self.selected_accounts.get_mut(index) {
                    account.name = account_name
                }
            }
            Message::Complete => {}
            Message::Next => return Ok(self.next(appdata)),
            Message::Back => self.back(),
        }

        Ok(Task::none())
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
                if self.password.len() > Password::MIN_LEN && self.password == self.verify_password
                {
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

        let create_wallet = Task::perform(
            async move {
                let (db_key, salt) = match &setup_data.db_key_salt {
                    Some(key_and_salt) => key_and_salt.clone(),
                    None => setup_data
                        .password
                        .derive_new_db_encryption_key()
                        .map_err(|err| AppError::Fatal(err.to_string()))?,
                };

                let (mnemonic_key, salt) = match &setup_data.mnemonic_key_salt {
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
                    db_key,
                    (mnemonic_key, salt),
                    &setup_data.selected_accounts,
                    network,
                )
                .await?;
                Ok::<_, AppError>(())
            },
            |result| match result {
                Ok(_) => Message::Complete.into(),
                Err(err) => AppMessage::Error(ErrorMessage::Fatal(err.to_string())).into(),
            },
        );

        let accounts_update =
            std::mem::replace(&mut self.accounts_update, AccountsUpdate::new(network));
        let icons = std::mem::take(&mut self.icons);
        let mut appdata = unsafe { MutUr::new(appdata) };
        let move_accounts_and_resources_to_appdata = Task::perform(
            async move {
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
            },
            |_| AppMessage::None,
        );

        Task::batch([create_wallet, move_accounts_and_resources_to_appdata])
    }

    pub fn view(&self, app: &'a App) -> Element<'a, AppMessage> {
        column!().into()
    }
}
