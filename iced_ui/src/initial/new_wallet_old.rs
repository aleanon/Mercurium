// use bip39::{Language, Mnemonic, MnemonicType};
// use debug_print::debug_println;
// use iced::widget::text::LineHeight;
// use iced::Task;
// use iced::{
//     widget::{self, text_input::Id, Button, Column, Row},
//     Element, Length,
// };
// use store::{AppDataDb, DataBase};
// use types::crypto::{EncryptedMnemonic, KeySaltPair, Password, SeedPhrase};
// use types::UnsafeRefMut;
// use types::{AppError, Network};
// use zeroize::Zeroize;

// use crate::app::AppData;
// use crate::external_task_response;
// use crate::{app::App, app::AppMessage};

// use super::setup;

// const INVALID_PASSWORD_LENGTH: &str = "Password must be between 16 and 64 characters long";
// const NON_ASCII_CHARACTERS: &str = "Password contains invalid characters";
// const EMPTY_ACCOUNT_NAME: &str = "Account name can not be empty";
// const MINIMUM_PASSWORD_LENGTH: usize = 16;
// const MAXIMUM_PASSWORD_LENGTH: usize = 64;

// #[derive(Debug, Clone)]
// pub enum Message {
//     // Back,
//     UpdatePassword(String),
//     SubmitPassword,
//     UpdateVerificationPassword(String),
//     VerifiPassword,
//     UpdateAccName(String),
//     SubmitAccName,
//     SeedPhrase,
//     VerifySeedPhrase,
//     InputSeedWord((usize, String)),
//     PasteInputSeed((usize, Vec<String>)),
//     Finalize,
// }

// impl Into<AppMessage> for Message {
//     fn into(self) -> AppMessage {
//         AppMessage::Setup(setup::Message::NewWalletMessage(self))
//     }
// }

// #[derive(Debug)]
// pub enum NewWalletStage {
//     EnterPassword,
//     VerifyPassword,
//     EnterAccountName,
//     EnterSeedPhrase,
//     ViewSeedPhrase,
//     VerifySeedPhrase,
// }

// #[derive(Debug)]
// pub struct NewWallet {
//     pub stage: NewWalletStage,
//     pub notification: &'static str,
//     pub password: Password,
//     pub verify_password: Password,
//     pub account_name: String,
//     pub mnemonic: Option<Mnemonic>,
//     pub seed_phrase: SeedPhrase,
// }

// impl<'a> NewWallet {
//     pub fn update(
//         &mut self,
//         message: Message,
//         app_data: &'a mut AppData,
//     ) -> Result<Task<AppMessage>, AppError> {
//         match message {
//             // Self::Back => Self::move_to_previous_step(new_wallet_state),
//             Message::UpdatePassword(mut input) => self.password_input(&mut input),
//             Message::SubmitPassword => self.submit_password(),
//             Message::UpdateVerificationPassword(input) => self.verify_password_input(input),
//             Message::VerifiPassword => self.verify_password(),
//             Message::UpdateAccName(input) => self.update_account_name_input(input),
//             Message::SubmitAccName => self.submit_account_name(),
//             Message::SeedPhrase => self.go_to_show_seed_phrase(),
//             Message::VerifySeedPhrase => self.go_to_verify_seed_phrase(),
//             Message::InputSeedWord((word_index, input)) => self.input_seed_word(word_index, input),
//             Message::PasteInputSeed((index, words)) => self.paste_input_seed(index, words),
//             Message::Finalize => return Ok(self.create_wallet(app_data)),
//         }
//         Ok(Task::none())
//     }

//     // fn move_to_previous_step(new_wallet_state: &'a mut NewWallet) -> Command<Message> {
//     //     match new_wallet_state.stage {
//     //         NewWalletStage::EnterPassword => {
//     //             return Command::
//     //         }
//     //         NewWalletStage::VerifyPassword => {
//     //             new_wallet_state.stage = NewWalletStage::EnterPassword;
//     //             new_wallet_state.verify_password.clear();
//     //             new_wallet_state.notification = "";
//     //         }
//     //         NewWalletStage::EnterAccountName => {
//     //             new_wallet_state.stage = NewWalletStage::EnterPassword;
//     //             new_wallet_state.password.clear();
//     //             new_wallet_state.verify_password.clear();
//     //             new_wallet_state.notification = "";
//     //         }
//     //         NewWalletStage::EnterSeedPhrase => {
//     //             new_wallet_state.stage = NewWalletStage::EnterAccountName;
//     //             new_wallet_state.mnemonic = None;
//     //             new_wallet_state.notification = "";
//     //         }
//     //         NewWalletStage::ViewSeedPhrase => {
//     //             new_wallet_state.stage = NewWalletStage::EnterAccountName;
//     //             new_wallet_state.notification = "";
//     //         }
//     //         NewWalletStage::VerifySeedPhrase => {
//     //             new_wallet_state.stage = NewWalletStage::ViewSeedPhrase;
//     //             new_wallet_state.notification = "";
//     //             new_wallet_state.seed_phrase = SeedPhrase::new();
//     //         }
//     //     }

//     //     Command::none()
//     // }

//     fn password_input(&mut self, input: &mut String) {
//         self.password.clear();
//         self.password.push_str(input.as_str());
//         input.zeroize();
//     }

//     fn submit_password(&mut self) {
//         if !self.password.as_str().is_ascii() {
//             self.notification = NON_ASCII_CHARACTERS
//         } else if self.password.as_str().len() < MINIMUM_PASSWORD_LENGTH
//             || self.password.as_str().len() > MAXIMUM_PASSWORD_LENGTH
//         {
//             self.notification = INVALID_PASSWORD_LENGTH
//         } else {
//             self.stage = NewWalletStage::VerifyPassword;
//             self.notification = "";
//         }
//     }

//     fn verify_password_input(&mut self, mut input: String) {
//         self.verify_password.clear();
//         self.verify_password.push_str(input.as_str());
//         input.zeroize();
//     }

//     fn verify_password(&mut self) {
//         if self.verify_password.as_str() == self.password.as_str() {
//             self.stage = NewWalletStage::EnterAccountName;
//             self.notification = "";
//         } else {
//             self.notification = "Password does not match";
//         }
//     }

//     fn update_account_name_input(&mut self, input: String) {
//         self.account_name = input;
//     }

//     fn submit_account_name(&mut self) {
//         if self.account_name.len() == 0 {
//             self.notification = EMPTY_ACCOUNT_NAME;
//         } else {
//             match self.mnemonic {
//                 Some(_) => self.stage = NewWalletStage::ViewSeedPhrase,
//                 None => self.stage = NewWalletStage::EnterSeedPhrase,
//             }
//             self.notification = "";
//         }
//     }

//     fn go_to_show_seed_phrase(&mut self) {
//         self.stage = NewWalletStage::ViewSeedPhrase;
//         self.notification = "";
//     }

//     fn go_to_verify_seed_phrase(&mut self) {
//         self.stage = NewWalletStage::VerifySeedPhrase;
//         self.notification = "";
//     }

//     fn input_seed_word(&mut self, word_index: usize, mut input: String) {
//         self.seed_phrase.update_word(word_index, input.as_str());
//         input.zeroize()
//     }

//     fn paste_input_seed(&mut self, mut index: usize, words: Vec<String>) {
//         for mut word in words {
//             self.seed_phrase.update_word(index, &word);
//             word.zeroize();
//             index += 1;
//         }
//     }

//     fn create_wallet(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
//         let network = appdata.settings.network.clone();

//         let mut new_wallet = unsafe { UnsafeRefMut::new(self) };
//         let mut appdata = unsafe { UnsafeRefMut::new(appdata) };
//         Task::perform(
//             async move {
//                 if let None = new_wallet.mnemonic {
//                     let phrase = new_wallet.seed_phrase.phrase();
//                     let mnemonic =
//                         match Mnemonic::from_phrase(phrase.as_str(), bip39::Language::English) {
//                             Ok(mnemonic) => mnemonic,
//                             Err(_) => {
//                                 new_wallet.notification = "Invalid seed phrase";
//                                 return Err(AppError::NonFatal(types::Notification::None));
//                             }
//                         };
//                     new_wallet.mnemonic = Some(mnemonic)
//                 }

//                 let mnemonic = new_wallet.mnemonic.as_ref().unwrap_or_else(|| {
//                     unreachable!("{}:{} Mnemonic not found", module_path!(), line!())
//                 });

//                 let (key, salt) = match KeySaltPair::<DataBase>::new(new_wallet.password.as_str()) {
//                     Ok(key_and_salt) => key_and_salt.into_inner(),
//                     Err(_) => {
//                         new_wallet.notification =
//                             "Unable to create random value for key derivation, please try again";
//                         return Err(AppError::NonFatal(types::Notification::None));
//                     }
//                 };

//                 let db = AppDataDb::load(network, key)
//                     .await
//                     .map_err(|err| AppError::Fatal(err.to_string()))?;

//                 let key_hash = new_wallet
//                     .password
//                     .derive_db_encryption_key_hash_from_salt(&salt);

//                 handles::credentials::store_db_encryption_salt(salt)
//                     .map_err(|err| AppError::Fatal(err.to_string()))?;

//                 db.upsert_password_hash(key_hash)
//                     .await
//                     .map_err(|err| AppError::Fatal(err.to_string()))?;

//                 let account = handles::wallet::create_account_from_mnemonic(
//                     mnemonic,
//                     None,
//                     0,
//                     0,
//                     new_wallet.account_name.clone(),
//                     Network::Mainnet,
//                 );

//                 debug_println!("Account created");

//                 appdata
//                     .accounts
//                     .insert(account.address.clone(), account.clone());

//                 db.upsert_account(account)
//                     .await
//                     .map_err(|err| AppError::Fatal(err.to_string()))?;

//                 debug_println!("Account stored in database");

//                 EncryptedMnemonic::new(mnemonic, "", &new_wallet.password)
//                     .map_err(|err| AppError::Fatal(err.to_string()))
//                     .and_then(|encrypted_mnemonic| {
//                         handles::credentials::store_encrypted_mnemonic(&encrypted_mnemonic)
//                     })?;

//                 Ok(())
//             },
//             |result| match result {
//                 Ok(_) => external_task_response::Message::WalletCreated.into(),
//                 Err(err) => external_task_response::Message::Error(err).into(),
//             },
//         )
//     }

//     pub fn new_with_mnemonic() -> Self {
//         Self {
//             stage: NewWalletStage::EnterPassword,
//             notification: "",
//             password: Password::new(),
//             verify_password: Password::new(),
//             account_name: String::new(),
//             mnemonic: Some(Mnemonic::new(MnemonicType::Words24, Language::English)),
//             seed_phrase: SeedPhrase::new(),
//         }
//     }

//     pub fn new_without_mnemonic() -> Self {
//         Self {
//             stage: NewWalletStage::EnterPassword,
//             notification: "",
//             password: Password::new(),
//             verify_password: Password::new(),
//             account_name: String::new(),
//             mnemonic: None,
//             seed_phrase: SeedPhrase::new(),
//         }
//     }
// }

// impl<'a> NewWallet {
//     pub fn view(&self, _app: &'a App) -> Element<'a, AppMessage> {
//         let content = match self.stage {
//             NewWalletStage::EnterPassword => self.enter_password_pane(),
//             NewWalletStage::VerifyPassword => self.verify_password_pane(),
//             NewWalletStage::EnterAccountName => self.account_name_pane(),
//             NewWalletStage::EnterSeedPhrase => self.enter_seed_phrase_pane(),
//             NewWalletStage::ViewSeedPhrase => self.view_seed_phrase(),
//             NewWalletStage::VerifySeedPhrase => self.enter_seed_phrase_pane(),
//         };

//         widget::container(content)
//             .center_x(660)
//             .center_y(700)
//             .into()
//     }

//     fn enter_password_pane(&self) -> Column<'a, AppMessage> {
//         let notification = Self::notification_field(self.notification);

//         let password_input = Self::text_input_field("Enter Password", &self.password.as_str())
//             .on_submit(Message::SubmitPassword.into())
//             .on_paste(|input| Message::UpdatePassword(input).into())
//             .on_input(|input| Message::UpdatePassword(input).into())
//             .secure(true);

//         let back = Self::nav_button("Back").on_press(setup::Message::Back.into());

//         let next = Self::nav_button("Next").on_press(Message::SubmitPassword.into());

//         let nav = Self::nav_row(back, next);

//         widget::column![notification, password_input, nav]
//             .align_x(iced::Alignment::Center)
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .spacing(50)
//     }

//     fn verify_password_pane(&self) -> Column<'a, AppMessage> {
//         let notification = Self::notification_field(self.notification);

//         let password_input =
//             Self::text_input_field("Verify Password", &self.verify_password.as_str())
//                 .on_submit(Message::VerifiPassword.into())
//                 .on_paste(|input| Message::UpdateVerificationPassword(input).into())
//                 .on_input(|input| Message::UpdateVerificationPassword(input).into())
//                 .secure(true);

//         let back = Self::nav_button("Back").on_press(setup::Message::Back.into());

//         let next = Self::nav_button("Next").on_press(Message::VerifiPassword.into());

//         let nav = Self::nav_row(back, next);

//         widget::column![notification, password_input, nav]
//             .align_x(iced::Alignment::Center)
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .spacing(50)
//     }

//     fn account_name_pane(&self) -> Column<'a, AppMessage> {
//         let notification = Self::notification_field(self.notification);

//         let account_name = Self::text_input_field("Enter account name", &self.account_name)
//             .on_submit(Message::SubmitAccName.into())
//             .on_input(|input| Message::UpdateAccName(input).into());

//         let back = Self::nav_button("Back").on_press(setup::Message::Back.into());

//         let next = Self::nav_button("Next").on_press(Message::SubmitAccName.into());

//         let nav = Self::nav_row(back, next);

//         widget::column![notification, account_name, nav]
//             .align_x(iced::Alignment::Center)
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .spacing(50)
//     }

//     fn view_seed_phrase(&self) -> Column<'a, AppMessage> {
//         let mut seed = widget::column![]
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .spacing(20);
//         let mut row = widget::row![]
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .spacing(20);

//         let seed_phrase = match self.mnemonic {
//             Some(ref mnemonic) => mnemonic.phrase(),
//             None => "",
//         };

//         for (i, word) in seed_phrase.split_ascii_whitespace().enumerate() {
//             if (i) % 4 == 0 && i != 0 {
//                 seed = seed.push(row);
//                 row = widget::row![]
//                     .width(Length::Shrink)
//                     .height(Length::Shrink)
//                     .spacing(20);
//             }

//             let text_field = Self::seed_word_field("", word).on_input(|mut string| {
//                 string.zeroize();
//                 AppMessage::None
//             });

//             row = row.push(text_field);
//         }
//         seed = seed.push(row);

//         let back = Self::nav_button("Back").on_press(setup::Message::Back.into());

//         let next = Self::nav_button("Next").on_press(Message::VerifySeedPhrase.into());

//         let nav = Self::nav_row(back, next);

//         widget::column![seed, nav]
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .spacing(50)
//     }

//     fn enter_seed_phrase_pane(&self) -> Column<'a, AppMessage> {
//         let input_seed = self.input_seed();

//         let back = Self::nav_button("Back").on_press(setup::Message::Back.into());

//         let next = Self::nav_button("Next").on_press(Message::Finalize.into());

//         let nav = Self::nav_row(back, next);

//         widget::column![input_seed, nav]
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .align_x(iced::Alignment::Center)
//             .spacing(50)
//     }

//     fn notification_field(text: &str) -> widget::Text {
//         widget::text(text).size(16).width(250)
//     }

//     fn text_input_field(placeholder: &str, input: &str) -> widget::TextInput<'a, AppMessage> {
//         widget::text_input(placeholder, input)
//             .size(16)
//             .width(250)
//             .line_height(LineHeight::Relative(1.5))
//     }

//     fn seed_word_field(placeholder: &str, input: &str) -> widget::TextInput<'a, AppMessage> {
//         widget::text_input(placeholder, input)
//             .size(16)
//             .width(100)
//             .line_height(LineHeight::Relative(2.))
//     }

//     fn input_seed(&self) -> Column<'a, AppMessage> {
//         let mut seed = widget::column![]
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .spacing(20);
//         let mut row = widget::row![]
//             .width(Length::Shrink)
//             .height(Length::Shrink)
//             .spacing(20);

//         for i in 0..24 {
//             if i % 4 == 0 && i != 0 {
//                 seed = seed.push(row);
//                 row = widget::row![]
//                     .width(Length::Shrink)
//                     .height(Length::Shrink)
//                     .spacing(20);
//             }
//             let mut word = "";

//             if let Some(s) = self.seed_phrase.reference_word(i) {
//                 word = s
//             }

//             let text_field = Self::seed_word_field(&format!("Word {}", i + 1), word)
//                 .id(Id::new(format!("{i}")))
//                 .on_input(move |input| Message::InputSeedWord((i, input)).into())
//                 .on_paste(move |mut string| {
//                     let i = i;
//                     let input = string
//                         .split_ascii_whitespace()
//                         .map(|s| String::from(s))
//                         .collect::<Vec<String>>();
//                     string.zeroize();
//                     Message::PasteInputSeed((i, input)).into()
//                 });

//             row = row.push(text_field);
//         }
//         seed.push(row)
//     }

//     fn nav_button(text: &'a str) -> Button<'a, AppMessage> {
//         Button::new(
//             widget::text(text)
//                 .size(16)
//                 .width(50)
//                 .align_x(iced::alignment::Horizontal::Center)
//                 .align_y(iced::alignment::Vertical::Center),
//         )
//     }

//     pub fn nav_row(
//         back: Button<'a, AppMessage>,
//         next: Button<'a, AppMessage>,
//     ) -> Row<'a, AppMessage> {
//         let space = widget::Space::with_width(Length::Fill);
//         widget::row![back, space, next]
//             .width(Length::Fill)
//             .align_y(iced::Alignment::Start)
//     }
// }
