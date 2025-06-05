use deps::*;

use iced::{Element, Task};
use types::address::AccountAddress;
use wallet::{Unlocked, Wallet};

use crate::{app::AppMessage, unlocked::app_view};

use super::{add_account::AddAccount, receive::Receive};

#[derive(Debug, Clone)]
pub enum Message {
    AddAccountMessage(super::add_account::Message),
    ReceiveMessage(super::receive::Message),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::OverlayMessage(self))
    }
}

#[derive(Debug, Clone)]
pub enum SpawnOverlay {
    AddAccount,
    Receive(AccountAddress),
}

#[derive(Debug, Clone)]
pub enum Overlay {
    AddAccount(AddAccount),
    Receive(Receive),
}

impl<'a> Overlay {
    pub fn update(&mut self, message: Message, wallet: &mut Wallet<Unlocked>) -> Task<AppMessage> {
        let mut command = Task::none();
        match message {
            Message::AddAccountMessage(message) => {
                if let Self::AddAccount(add_account) = self {
                    command = add_account.update(message, wallet)
                }
            }
            Message::ReceiveMessage(message) => {
                if let Self::Receive(receive) = self {
                    command = receive.update(message)
                }
            }
        }
        command
    }

    pub fn view(&'a self, _wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        match self {
            Self::AddAccount(add_account_view) => add_account_view.view(),
            Self::Receive(receive) => receive.view(),
        }
    }
}
