pub mod add_account_message;
pub mod receive;

use iced::Command;

use crate::{message::Message, App};

use self::{add_account_message::AddAccountMessage, receive::ReceiveMessage};

use super::AppViewMessage;

#[derive(Debug, Clone)]
pub enum OverlayMessage {
    AddAccountMessage(AddAccountMessage),
    ReceiveMessage(ReceiveMessage),
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
            Self::ReceiveMessage(receive_message) => receive_message.update(app),
        }
    }
}
