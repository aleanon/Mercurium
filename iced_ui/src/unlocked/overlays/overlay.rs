use iced::{Command, Element};

use crate::{app::AppData, app::AppMessage, unlocked::app_view};

use super::{add_account::AddAccountView, receive::Receive};

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
pub enum Overlay {
    AddAccount(AddAccountView),
    Receive(Receive),
}

impl<'a> Overlay {
    pub fn update(&mut self, message: Message, appdata: &mut AppData) -> Command<AppMessage> {
        let mut command = Command::none();
        match message {
            Message::AddAccountMessage(message) => {
                if let Self::AddAccount(add_account) = self {
                    command = add_account.update(message, appdata)
                }
            }
            Message::ReceiveMessage(message) => {
                if let Self::Receive(receive) = self {
                    command = receive.update(message, appdata)
                }
            }
        }
        command
    }

    pub fn view(&'a self, appdata: &'a AppData) -> Element<'a, AppMessage> {
        match self {
            Self::AddAccount(add_account_view) => add_account_view.view(appdata),
            Self::Receive(receive) => receive.view(appdata),
        }
    }
}
