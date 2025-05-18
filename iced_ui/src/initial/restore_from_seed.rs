use deps::{debug_print::debug_println, *};

use iced::{widget::{self, container}, Element, Length, Task};
use types::{AppError, Notification};
use wallet::{wallet::Wallet, Setup, Unlocked};

use super::pages::{choose_accounts::{self, ChooseAccounts}, enter_password::{self, EnterPassword}, enter_seed_phrase::{self, EnterSeedPhrase}, name_accounts::{self, NameAccounts}};

pub enum Action {
    SetupSelection,
    Task(Task<Message>),
    None,
}

#[derive(Clone)]
pub enum Message {
    SelectSetup,
    Back,
    Next,
    EnterSeedPhrase(Notification),
    EnterPassword(Notification),
    ChooseAccounts,
    NameAccounts,
    EnterSeedPhraseMessage(enter_seed_phrase::Message),
    EnterPasswordMessage(enter_password::Message),
    ChooseAccountsMessage(choose_accounts::Message),
    NameAccountsMessage(name_accounts::Message),
    WalletCreated(Wallet<Unlocked>),
    Error(AppError)
}

#[derive(Debug)]
pub enum RestoreFromSeed {
    EnterSeedPhrase(EnterSeedPhrase),
    EnterPassword(EnterPassword),
    ChooseAccounts(ChooseAccounts),
    NameAccounts(NameAccounts),
    Finalizing,
}

impl<'a> RestoreFromSeed {
    pub fn new(wallet: &mut Wallet<Setup>) -> Self {
        RestoreFromSeed::EnterSeedPhrase(EnterSeedPhrase::new(wallet, Notification::None))
    }

    pub fn update(&mut self, message: Message, wallet: &'a mut Wallet<Setup>) -> Action {
        match message {
            Message::EnterSeedPhraseMessage(enter_seed_phrase::Message::Back)
            | Message::EnterPasswordMessage(enter_password::Message::Back)
            | Message::ChooseAccountsMessage(choose_accounts::Message::Back)
            | Message::NameAccountsMessage(name_accounts::Message::Back) => return self.back(wallet),
            Message::EnterSeedPhraseMessage(enter_seed_phrase::Message::Next)
            | Message::EnterPasswordMessage(enter_password::Message::Next)
            | Message::ChooseAccountsMessage(choose_accounts::Message::Next)
            | Message::NameAccountsMessage(name_accounts::Message::Next) => return self.next(wallet),
            Message::EnterSeedPhraseMessage(message) => {
                if let Self::EnterSeedPhrase(page) = self {
                    return Action::Task(page.update(message, wallet)
                        .map(Message::EnterSeedPhraseMessage))
                }
            }
            Message::EnterPasswordMessage(message) => {
                if let Self::EnterPassword(page) = self {
                    return Action::Task(page.update(message, wallet)
                        .map(Message::EnterPasswordMessage))
                }
            }
            Message::ChooseAccountsMessage(message) => {
                if let Self::ChooseAccounts(page) = self {
                    return Action::Task(page.update(message, wallet)
                        .map(Message::ChooseAccountsMessage))
                }
            }
            Message::NameAccountsMessage(message) => {
                if let Self::NameAccounts(page) = self {
                    return Action::Task(page.update(message)
                        .map(Message::NameAccountsMessage))
                }
            }
            Message::Back => return self.back(wallet),
            Message::Next => return self.next(wallet),
            Message::WalletCreated(_) => {},
            Message::SelectSetup => {},
            Message::Error(_) => {},
            Message::EnterSeedPhrase(notification) => {
                *self = Self::EnterSeedPhrase(EnterSeedPhrase::new(wallet, notification))
            }
            Message::EnterPassword(notification) => {
                *self = Self::EnterPassword(EnterPassword::new(wallet, notification))
            }
            Message::ChooseAccounts => {
                let (page, task) = ChooseAccounts::new(wallet);
                *self = Self::ChooseAccounts(page);
                return Action::Task(task.map(Message::ChooseAccountsMessage))
            }
            Message::NameAccounts => {
                *self = Self::NameAccounts(NameAccounts::new(wallet))
            }
        }
        Action::None
    }

    fn back(&mut self, wallet: &Wallet<Setup>) -> Action {
        match self {
            Self::EnterSeedPhrase(_) => return Action::SetupSelection,
            Self::EnterPassword(_) => *self = Self::EnterSeedPhrase(EnterSeedPhrase::new(wallet, Notification::None)),
            Self::ChooseAccounts(_) => *self = Self::EnterPassword(EnterPassword::new(wallet, Notification::None)),
            Self::NameAccounts(_) => {
                let (page, task) = ChooseAccounts::new(wallet);
                *self = Self::ChooseAccounts(page);
                return Action::Task(task.map(Message::ChooseAccountsMessage))
            }
            Self::Finalizing => {},
        }
        Action::None
    }

    fn next(&mut self, wallet: &'a mut Wallet<Setup>) -> Action {
        match self {
            Self::EnterSeedPhrase(page) => {
                if let Err(_) = page.save_to_wallet(wallet) {
                    page.notification = Notification::Info("Invalid seed phrase".to_string())
                } else {
                    *self = Self::EnterPassword(EnterPassword::new(wallet, Notification::None));
                }
            },
            Self::EnterPassword(page) => {
                if let Err(_) = page.save_to_wallet(wallet) {
                    return Action::None
                }
                let (page, task) = ChooseAccounts::new(wallet);
                *self = Self::ChooseAccounts(page);
                return Action::Task(task.map(Message::ChooseAccountsMessage))
            },
            Self::ChooseAccounts(page) => {
                page.save_to_wallet(wallet);
                *self = Self::NameAccounts(NameAccounts::new(wallet))
            },
            Self::NameAccounts(page) => {
                page.save_to_wallet(wallet);
                *self = Self::Finalizing;
                let setup = wallet.get_setup();
                return Action::Task(Task::perform(async move {
                    let result = setup.finalize_setup().await
                        .inspect_err(|err| println!("{}", err.to_string()));
                    debug_println!("Setup finished");
                    result
                }, |result| {
                    match result {
                        Ok(wallet) => Message::WalletCreated(wallet),
                        Err(err) => Message::Error(AppError::Fatal(err.to_string()))
                    }
                }));
            }
            Self::Finalizing => {},
        }
        Action::None
    }
}


impl<'a> RestoreFromSeed {
    pub fn view(&'a self) -> Element<'a, Message> {
        let page = match self {
            Self::EnterSeedPhrase(page) => page.view()
                .map(Message::EnterSeedPhraseMessage),
            Self::EnterPassword(page) => page.view()
                .map(Message::EnterPasswordMessage),
            Self::ChooseAccounts(page) => page.view()
                .map(Message::ChooseAccountsMessage),
            Self::NameAccounts(page) => page.view()
                .map(Message::NameAccountsMessage),
            Self::Finalizing => widget::text("Finalizing setup...").into(),
        };

        container(page)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into() 
    }
}