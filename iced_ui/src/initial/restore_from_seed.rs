use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::Display,
    sync::Arc,
};

use bip39::Mnemonic;
use iced::{
    widget::{
        self, column, image::Handle, row, text::LineHeight, text_input::Id, Button, Column, Row,
    },
    Element, Length, Task,
};
use types::{
    address::{AccountAddress, Address, ResourceAddress},
    assets::{FungibleAsset, NonFungibleAsset},
    collections::AccountsUpdate,
    crypto::{DataBaseKey, Key, Password, PasswordError, Salt, SeedPhrase},
    Account, AppError, AppSettings, MutUr, Network, Ur,
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
    AccountsUpdated((AppData, BTreeMap<ResourceAddress, String>)),
    IconsReceived(HashMap<ResourceAddress, Handle>),
    NewPage(usize),
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
                        return Ok(Task::perform(
                            async move {
                                let accounts_update = handles::radix_dlt::updates::update_accounts(
                                    network,
                                    Arc::new(HashMap::new()),
                                    accounts,
                                )
                                .await;

                                let mut accounts: HashMap<AccountAddress, Account> = HashMap::new();
                                let mut fungible_tokens: HashMap<
                                    AccountAddress,
                                    BTreeSet<FungibleAsset>,
                                > = HashMap::new();
                                let mut non_fungible_tokens: HashMap<
                                    AccountAddress,
                                    BTreeSet<NonFungibleAsset>,
                                > = HashMap::new();

                                for account_update in accounts_update.account_updates {
                                    let fungibles =
                                        account_update.fungibles.into_values().collect();

                                    fungible_tokens
                                        .insert(account_update.account.address.clone(), fungibles);

                                    let non_fungibles =
                                        account_update.non_fungibles.into_values().collect();

                                    non_fungible_tokens.insert(
                                        account_update.account.address.clone(),
                                        non_fungibles,
                                    );

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
                        ));
                    }
                }
            }
            Message::InputPassword(mut input) => {
                self.password.clear();
                self.password.push_str(input.as_str());
                input.zeroize();
                self.notification = "";
            }
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

    pub fn view(&'a self, _app: &'a App) -> Element<'a, AppMessage> {
        let content = match self.stage {
            Stage::EnterSeedPhrase => self.enter_seed_phrase_view(),
            Stage::EnterPassword => self.enter_password_view(),
            Stage::ChooseAccounts => self.choose_accounts_view(),
            Stage::NameAccounts => column!().into(),
            Stage::Finalizing => column!().into(),
        };

        widget::container(content)
            .center_x(660)
            .center_y(700)
            .into()
    }

    fn enter_seed_phrase_view(&self) -> Column<'a, AppMessage> {
        let input_seed = self.input_seed();

        let nav = Self::nav_row(
            Self::nav_button("Back").on_press(Message::Back.into()),
            Self::nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![input_seed, nav]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_items(iced::Alignment::Center)
            .spacing(50)
    }

    fn enter_password_view(&self) -> Column<'a, AppMessage> {
        let password_notification = Self::notification_field(self.notification);

        let password_input = Self::text_input_field("Enter Password", &self.password.as_str())
            .on_paste(|input| Message::InputPassword(input).into())
            .on_input(|input| Message::InputPassword(input).into())
            .secure(true);

        let verify_pw_input =
            Self::text_input_field("Verify Password", &self.verify_password.as_str())
                .on_submit(Message::Next.into())
                .on_paste(|input| Message::InputVerifyPassword(input).into())
                .on_input(|input| Message::InputVerifyPassword(input).into())
                .secure(true);

        let nav = Self::nav_row(
            Self::nav_button("Back").on_press(Message::Back.into()),
            Self::nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![password_notification, password_input, verify_pw_input, nav]
            .align_items(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }

    fn choose_accounts_view(&'a self) -> Column<'a, AppMessage> {
        let mut accounts = column!().height(400);

        let page = self.accounts.get(self.page_index);
        if let Some(accounts_selection) = page {
            for (i, (account, is_selected, account_summary)) in
                accounts_selection.iter().enumerate()
            {
                let account_address = widget::text(account.address.truncate());
                let account_summary = widget::text(account_summary.to_string());

                let is_selected = widget::checkbox("", *is_selected).on_toggle(move |_| {
                    Message::ToggleAccountSelection((self.page_index, i)).into()
                });

                accounts = accounts.push(
                    row![
                        account_address.width(Length::FillPortion(10)),
                        widget::Space::new(Length::Fill, 1),
                        account_summary.width(Length::FillPortion(10)),
                        widget::Space::new(Length::FillPortion(2), 1),
                        is_selected.width(Length::FillPortion(2))
                    ]
                    .width(Length::Fill),
                )
            }
        }

        let row = row![
            widget::button("Previous Page").on_press_maybe(if self.page_index == 0 {
                None
            } else {
                Some(Message::NewPage(self.page_index - 1).into())
            }),
            accounts.width(400),
            widget::button("Next Page").on_press(Message::NewPage(self.page_index + 1).into())
        ]
        .align_items(iced::Alignment::Center);

        let nav_buttons = Self::nav_row(
            Self::nav_button("Back").on_press(Message::Back.into()),
            Self::nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![row, nav_buttons]
            .align_items(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }

    fn notification_field(text: &str) -> widget::Text {
        widget::text(text).size(16).width(250)
    }

    fn text_input_field(placeholder: &str, input: &str) -> widget::TextInput<'a, AppMessage> {
        widget::text_input(placeholder, input)
            .size(16)
            .width(250)
            .line_height(LineHeight::Relative(1.5))
    }

    fn seed_word_field(placeholder: &str, input: &str) -> widget::TextInput<'a, AppMessage> {
        widget::text_input(placeholder, input)
            .size(16)
            .width(100)
            .line_height(LineHeight::Relative(2.))
    }

    fn input_seed(&self) -> Column<'a, AppMessage> {
        let mut seed = widget::column![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);
        let mut row = widget::row![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);

        for i in 0..24 {
            if i % 4 == 0 && i != 0 {
                seed = seed.push(row);
                row = widget::row![]
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(20);
            }
            let mut word = "";

            if let Some(s) = self.seed_phrase.reference_word(i) {
                word = s
            }

            let text_field = Self::seed_word_field(&format!("Word {}", i + 1), word)
                .id(Id::new(format!("{i}")))
                .on_input(move |input| Message::InputSeedWord((i, input)).into())
                .on_paste(move |mut string| {
                    let input = string
                        .split_ascii_whitespace()
                        .map(|s| String::from(s))
                        .collect::<Vec<String>>();
                    string.zeroize();
                    Message::PasteSeedPhrase((i, input)).into()
                });

            row = row.push(text_field);
        }
        seed.push(row)
    }

    fn nav_button(text: &'a str) -> Button<'a, AppMessage> {
        Button::new(
            widget::text(text)
                .size(16)
                .width(50)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
    }

    pub fn nav_row(
        back: Button<'a, AppMessage>,
        next: Button<'a, AppMessage>,
    ) -> Row<'a, AppMessage> {
        let space = widget::Space::with_width(Length::Fill);
        widget::row![back, space, next]
            .width(Length::Fill)
            .align_items(iced::Alignment::Start)
    }
}
