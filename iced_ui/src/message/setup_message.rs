pub mod new_wallet_update;
pub mod restore_wallet_update;

use iced::Command;
use types::crypto::SeedPhrase;

use crate::{app::{App, AppData, AppState}, view::setup::{new_wallet::{NewWallet, NewWalletStage}, Setup}}; 

use self::{new_wallet_update::WalletMessage, restore_wallet_update::RestoreWalletMessage};

use super::Message;





#[derive(Debug, Clone)]
pub enum SetupMessage {
    Back,
    Restore,
    FromSeed,
    NewWallet,
    NewWalletMessage(WalletMessage),
    RestoreWalletMessage(RestoreWalletMessage)
}

impl Into<Message> for SetupMessage {
    fn into(self) -> Message {
        Message::Setup(self)
    }
}

impl<'a> SetupMessage {
    pub fn process(self, setup: &'a mut Setup, app_data: &'a mut AppData) -> Command<Message> {
        let mut command = Command::none();
        match self {
            Self::Back => Self::back(setup),
            Self::NewWallet => {
                if let Setup::SelectCreation = setup {
                    *setup = Setup::NewWallet(NewWallet::new_with_mnemonic())
                }
            }
            Self::FromSeed => {
                *setup = Setup::NewWallet(NewWallet::new_without_mnemonic())
            }
            Self::NewWalletMessage(new_wallet) => {
                command = new_wallet.process(setup, app_data);
            }
            _ => {}
        }
        command
    } 

    fn back(setup: &'a mut Setup) {
        match setup {
            Setup::NewWallet(new_wallet_state) => {
                match new_wallet_state.stage {
                    NewWalletStage::EnterPassword => {
                        *setup = Setup::SelectCreation
                    }
                    NewWalletStage::VerifyPassword => {
                        new_wallet_state.stage = NewWalletStage::EnterPassword;
                        new_wallet_state.verify_password.clear();
                        new_wallet_state.notification = "";
                    }
                    NewWalletStage::EnterAccountName => {
                        new_wallet_state.stage = NewWalletStage::EnterPassword;
                        new_wallet_state.password.clear();
                        new_wallet_state.verify_password.clear();
                        new_wallet_state.notification = "";
                    }
                    NewWalletStage::EnterSeedPhrase => {
                        new_wallet_state.stage = NewWalletStage::EnterAccountName;
                        new_wallet_state.mnemonic = None;
                        new_wallet_state.notification = "";
                    }
                    NewWalletStage::ViewSeedPhrase => {
                        new_wallet_state.stage = NewWalletStage::EnterAccountName;
                        new_wallet_state.notification = "";
                    }
                    NewWalletStage::VerifySeedPhrase => {
                        new_wallet_state.stage = NewWalletStage::ViewSeedPhrase;
                        new_wallet_state.notification = "";
                        new_wallet_state.seed_phrase = SeedPhrase::new();
                    }
                }
            }
            _ => {}
        };
    }
}