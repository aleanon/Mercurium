pub mod new_wallet_update;
pub mod restore_wallet_update;

use iced::Command;

use crate::{app::{App, State}, view::setup::{new_wallet::NewWallet, Setup}}; 

use self::{new_wallet_update::WalletMessage, restore_wallet_update::RestoreWalletMessage};

use super::Message;





#[derive(Debug, Clone)]
pub enum SetupMessage {
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
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();
        match self {
            Self::NewWallet => {
                if let State::Initial(ref mut setup)  = app.state {
                    if let Setup::SelectCreation = setup {
                        *setup = Setup::NewWallet(NewWallet::new_with_mnemonic())
                    }
                }
            }
            Self::FromSeed => {
                if let State::Initial(ref mut setup) = app.state {
                    *setup = Setup::NewWallet(NewWallet::new_without_mnemonic())
                }
            }
            Self::NewWalletMessage(new_wallet) => {
                command = new_wallet.process(app);
            }
            _ => {}
        }
        command
    } 
}