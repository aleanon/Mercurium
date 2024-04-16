use iced::{clipboard, Command};
use types::crypto::Key;

use crate::{message::Message, App};

#[derive(Debug, Clone)]
pub enum CommonMessage {
    CopyToClipBoard(String),
    PerformLogin(Key),
    Notify(String),
}

impl Into<Message> for CommonMessage {
    fn into(self) -> Message {
        Message::Common(self)
    }
}

impl<'a> CommonMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();
        match self {
            Self::CopyToClipBoard(input) => command = clipboard::write(input),
            Self::Notify(message) => app.appview.notification = Some(message),
            Self::PerformLogin(_key) => {
                app.login();
            }
        };
        command
    }
}
