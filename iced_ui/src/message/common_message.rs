use iced::{clipboard, Command};
use types::crypto::Key;

use crate::{message::Message, App};


#[derive(Debug, Clone)]
pub enum CommonMessage {
    CopyToClipBoard(String),
    PerformLogin(Key),
}

impl<'a> CommonMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();
        match self {
            Self::CopyToClipBoard(input) => command = clipboard::write(input),
            Self::PerformLogin(_key) => {app.login();},
        };
        command
    }

    fn copy_to_clipboard(input: String) -> Command<Message> {
        iced::clipboard::write(input)
    }
}
