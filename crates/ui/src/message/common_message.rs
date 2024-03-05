use iced::Command;

use crate::{message::Message, App};


#[derive(Debug, Clone)]
pub enum CommonMessage {
    CopyToClipBoard(String),
}

impl<'a> CommonMessage {
    pub fn process(self, _app: &'a mut App) -> Command<Message> {
        match self {
            Self::CopyToClipBoard(input) => Self::copy_to_clipboard(input),
        }
    }

    fn copy_to_clipboard(input: String) -> Command<Message> {
        iced::clipboard::write(input)
    }
}
