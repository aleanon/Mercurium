use iced::{clipboard, Command};
use types::crypto::Key;

use crate::App;

use super::app::AppMessage;

#[derive(Debug, Clone)]
pub enum Message {
    CopyToClipBoard(String),
    PerformLogin(Key),
    Notify(String),
}

// impl Into<Message> for CommonMessage {
//     fn into(self) -> Message {
//         Message::Common(self)
//     }
// }

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Common(self)
    }
}

impl<'a> Message {
    pub fn process(self, app: &'a mut App) -> Command<AppMessage> {
        let mut command = Command::none();
        match self {
            Self::CopyToClipBoard(input) => command = clipboard::write(input),
            Self::Notify(message) => app.appview.notification = Some(message),
            Self::PerformLogin(_key) => {
                app.login().ok();
            }
        };
        command
    }
}
