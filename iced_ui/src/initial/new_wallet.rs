use deps::{iced::{widget, Length}, *};

use iced::{widget::text, Element, Task};
use types::{AppError, Notification};
use wallet::{SetupError, Unlocked, Wallet};

use super::page::{enter_password::{self, EnterPassword}, name_accounts::{self, NameAccounts}, verify_seed_phrase::{self, VerifySeedPhrase}, view_seed_phrase::{self, ViewSeedPhrase}};


#[derive(Clone)]
pub enum Message {
    SetupSelection,
    Back,
    Next,
    EnterPassword(Notification),
    ViewSeedPhrase,
    VerifySeedPhrase,
    NameAccount,
    Finalize,
    WalletCreated(Wallet<Unlocked>),
    Error(AppError),
    EnterPasswordMessage(enter_password::Message),
    ViewSeedPhraseMessage(view_seed_phrase::Message),
    VerifySeedPhraseMessage(verify_seed_phrase::Message),
    NameAccountMessage(name_accounts::Message),
}

#[derive(Debug)]
pub enum NewWallet {
    EnterPassword(EnterPassword),
    ViewSeedPhrase(ViewSeedPhrase),
    VerifySeedPhrase(VerifySeedPhrase),
    NameAccount(NameAccounts),
    Finalizing,
}


impl<'a> NewWallet {
    pub fn new(wallet: &mut Wallet<wallet::Setup>) -> Self {
        wallet.create_random_mnemonic();
        Self::EnterPassword(EnterPassword::new(wallet, Notification::None))
    }

    pub fn update(&mut self, message: Message, wallet: &'a mut Wallet<wallet::Setup>) -> Task<Message> {
        match message {
            Message::Back => self.back(wallet),
            Message::Next => return self.next(wallet),
            Message::EnterPassword(notification) => {
                self.save_current_page_to_wallet(wallet);
                *self = Self::EnterPassword(EnterPassword::new(wallet, notification));
            }
            Message::ViewSeedPhrase => {
                self.save_current_page_to_wallet(wallet);
                *self = Self::ViewSeedPhrase(ViewSeedPhrase::new(wallet, Notification::None));
                
            }
            Message::VerifySeedPhrase => {
                self.save_current_page_to_wallet(wallet);
                *self = Self::VerifySeedPhrase(VerifySeedPhrase::new(wallet, Notification::None))
            }
            Message::NameAccount => {
                self.save_current_page_to_wallet(wallet);
                *self = Self::NameAccount(NameAccounts::new(wallet))
            }
            Message::EnterPasswordMessage(message) => {
                if let Self::EnterPassword(page) = self {
                    return page.update(message, wallet).map(Message::EnterPasswordMessage)
                }
            }
            Message::ViewSeedPhraseMessage(message) => {
                if let Self::ViewSeedPhrase(page) = self {
                    return page.update(message, wallet).map(Message::ViewSeedPhraseMessage)
                }
            }
            Message::VerifySeedPhraseMessage(message) => {
                if let Self::VerifySeedPhrase(page ) = self {
                    return page.update(message, wallet).map(Message::VerifySeedPhraseMessage)
                };
            }
            Message::NameAccountMessage(message) => {
                if let Self::NameAccount(page) = self {
                    return page.update(message).map(Message::NameAccountMessage)
                }
            }
            Message::Finalize => {
                *self = Self::Finalizing;
                let setup = wallet.get_setup();
                return Task::perform(async move {
                    setup.finalize_setup().await
                }, |result| {
                    match result {
                        Ok(wallet) => Message::WalletCreated(wallet),
                        Err(err) => {
                            match err {
                                SetupError::MissingDerivedKeys => Message::EnterPassword(Notification::Info("Failed to derive keys, try a different password".to_string())),
                                _ => Message::Error(AppError::Fatal(err.to_string()))
                            }
                        }
                    }
                })
            }
            Message::SetupSelection => {/*Handled in parent*/}
            Message::WalletCreated(_) => {/*Propagated*/}
            Message::Error(_) => {/*Propagated*/}
        }
        Task::none()
    }

    fn back(&mut self, wallet: &mut Wallet<wallet::Setup>) {
        match self {
            Self::EnterPassword(_) => {/*Back to setup selection handled in parent*/}
            Self::ViewSeedPhrase(page) => {
                page.save_to_wallet(wallet).ok();
                *self = Self::EnterPassword(EnterPassword::new(wallet, Notification::None))
            },
            Self::VerifySeedPhrase(_) => *self = Self::ViewSeedPhrase(ViewSeedPhrase::new(wallet, Notification::None)),
            Self::NameAccount(page) => {
                page.save_to_wallet(wallet);
                *self = Self::VerifySeedPhrase(VerifySeedPhrase::new(wallet, Notification::None))
            }
            Self::Finalizing => {/*Back unavailable*/}
        }
    }

    fn next(&mut self, wallet: &mut Wallet<wallet::Setup>) -> Task<Message> {
        match self {
            Self::EnterPassword(page) => {
                if let Err(_) = page.save_to_wallet(wallet) {
                    return Task::none()
                }

                *self = Self::ViewSeedPhrase(ViewSeedPhrase::new(wallet, Notification::None));
            }
            Self::ViewSeedPhrase(_) => *self = Self::VerifySeedPhrase(VerifySeedPhrase::new(wallet, Notification::None)),
            Self::VerifySeedPhrase(page) => {
                if !page.seed_phrase_is_correct() {return Task::none()}
                *self = Self::NameAccount(NameAccounts::new(wallet))
            }
            Self::NameAccount(page) => {
                page.save_to_wallet(wallet);
                return self.finalize_setup(wallet)
            }
            Self::Finalizing => {/*Next unavailable*/}
        }
        Task::none()
    }

    fn save_current_page_to_wallet(&mut self, wallet: &mut Wallet<wallet::Setup>) -> Result<(), AppError> {
        match self {
            Self::EnterPassword(page) => {page.save_to_wallet(wallet).ok();},
            Self::ViewSeedPhrase(page) => return page.save_to_wallet(wallet),
            Self::VerifySeedPhrase(_) => {},
            _ => {},
        }
        Ok(())
    }

    fn finalize_setup(&mut self, wallet: &Wallet<wallet::Setup>) -> Task<Message>  {
        *self = Self::Finalizing;
        let setup = wallet.get_setup();
        return Task::perform(async move {
            setup.finalize_setup().await
        }, |result| {
            match result {
                Ok(wallet) => Message::WalletCreated(wallet),
                Err(err) => {
                    match err {
                        SetupError::MissingDerivedKeys => Message::EnterPassword(Notification::Info("Failed to derive keys, try a different password".to_string())),
                        _ => Message::Error(AppError::Fatal(err.to_string()))
                    }
                }
            }
        })
    }

    pub fn view(&'a self, wallet: &Wallet<wallet::Setup>) -> Element<'a, Message> {
        let page = match self {
            Self::EnterPassword(page) => page.view().map(|message| match message {
                enter_password::Message::Back => Message::SetupSelection,
                enter_password::Message::Next => Message::ViewSeedPhrase,
                m => Message::EnterPasswordMessage(m)
            }),
            Self::ViewSeedPhrase(page) => page.view().map(|message| match message {
                view_seed_phrase::Message::Back => Message::EnterPassword(Notification::None),
                view_seed_phrase::Message::Next => Message::VerifySeedPhrase,
                m => Message::ViewSeedPhraseMessage(m),
            }),
            Self::VerifySeedPhrase(page) => page.view().map(Message::VerifySeedPhraseMessage),
            Self::NameAccount(page) => page.view().map(Message::NameAccountMessage),
            Self::Finalizing => text("Setting up wallet...").into()
        };

        widget::container(page)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into() 
    }
}