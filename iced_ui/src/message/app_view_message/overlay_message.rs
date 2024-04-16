use iced::Command;

use crate::{message::Message, App};

use self::add_account_message::AddAccountMessage;

use super::AppViewMessage;

pub mod add_account_message;

#[derive(Debug, Clone)]
pub enum OverlayMessage {
    AddAccountMessage(AddAccountMessage),
}

impl Into<Message> for OverlayMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::OverlayMessage(self))
    }
}

impl OverlayMessage {
    pub fn update(self, app: &mut App) -> Command<Message> {
        match self {
            Self::AddAccountMessage(add_account_message) => add_account_message.update(app),
        }
    }
}
