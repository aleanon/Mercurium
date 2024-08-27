use bip39::Mnemonic;
use iced::{widget::column, Element, Task};
use types::{
    crypto::{Password, SeedPhrase},
    Account, AppError, Ur,
};
use zeroize::Zeroize;

use crate::{
    app::{AppData, AppMessage},
    App,
};

#[derive(Debug, Clone)]
pub enum Message {
    InputSeedWord((usize, String)),
    PasteSeedPhrase((usize, Vec<String>)),
    ToggleSeedPassword,
    InputSeedPassword(String),
    AccountsReceived(Vec<Account>),
    InputPassword(String),
    InputVerifyPassword(String),
    ToggleAccountSelection((usize, usize)),
    InputAccountName((usize, String)),
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

#[derive(Debug)]
pub struct AccountSummary {}

#[derive(Debug)]
pub struct RestoreFromSeed {
    pub stage: Stage,
    pub notification: &'static str,
    pub seed_phrase: SeedPhrase,
    pub seed_password: Option<Password>,
    pub mnemonic: Option<Mnemonic>,
    pub password: Password,
    pub verify_password: Password,
    pub accounts: Vec<Vec<(Account, bool, AccountSummary)>>,
    pub selected_accounts: Vec<Account>,
}

impl<'a> RestoreFromSeed {
    pub fn new() -> Self {
        Self {
            stage: Stage::EnterSeedPhrase,
            notification: "",
            seed_phrase: SeedPhrase::new(),
            seed_password: None,
            mnemonic: None,
            password: Password::new(),
            verify_password: Password::new(),
            accounts: Vec::new(),
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

                if let Ok(mnemonic) = Mnemonic::from_phrase(
                    self.seed_phrase.phrase().as_str(),
                    bip39::Language::English,
                ) {
                    self.mnemonic = Some(mnemonic);
                }
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
            Message::AccountsReceived(accounts) => {
                match self.stage {
                    Stage::EnterSeedPhrase => { /*If the user has gone back we want to drop this value*/
                    }
                    _ => { /*Create task to get account summary from radix gateway */ }
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
            Message::ToggleAccountSelection((chunk_index, account_index)) => {
                if let Some(chunk) = self.accounts.get_mut(chunk_index) {
                    if let Some((account, is_selected, summary)) = chunk.get_mut(account_index) {
                        *is_selected = !*is_selected
                    }
                }
            }
            Message::InputAccountName((index, account_name)) => {
                if let Some(account) = self.selected_accounts.get_mut(index) {
                    account.name = account_name
                }
            }
            Message::Next => return Ok(self.next(appdata)),
            Message::Back => self.back(appdata),
        }

        Ok(Task::none())
    }

    fn next(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        match self.stage {
            Stage::EnterSeedPhrase => {
                let Some(mnemonic) = &self.mnemonic else {
                    self.notification = "Error mnemonic phrase, try entering again";
                    self.mnemonic = None;
                    return Task::none();
                };
                self.stage = Stage::EnterPassword;

                let mnemonic = mnemonic.clone();
                let password = self.seed_password.clone();
                let network = appdata.settings.network;

                return Task::perform(
                    async move {
                        handles::wallet::create_multiple_accounts_from_mnemonic::<Vec<_>>(
                            &mnemonic,
                            password
                                .as_ref()
                                .and_then(|password| Some(password.as_str())),
                            0,
                            0,
                            60,
                            network,
                        )
                    },
                    |accounts| Message::AccountsReceived(accounts).into(),
                );
            }
        }

        Task::none()
    }

    fn back(&mut self, appdata: &'a mut AppData) {
        match self.stage {
            Stage::EnterSeedPhrase => {}
            Stage::EnterPassword => self.stage = Stage::EnterSeedPhrase,
            Stage::ChooseAccounts => self.stage = Stage::EnterPassword,
            Stage::NameAccounts => self.stage = Stage::ChooseAccounts,
            Stage::Finalizing => self.stage = Stage::NameAccounts,
        }
    }

    pub fn view(&self, appdata: &'a App) -> Element<'a, AppMessage> {
        column!().into()
    }
}
