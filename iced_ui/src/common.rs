use iced::{clipboard, Task};

use crate::App;

use super::app::AppMessage;

#[derive(Debug, Clone)]
pub enum Message {
    CopyToClipBoard(String),
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
    pub fn process(self, app: &'a mut App) -> Task<AppMessage> {
        let mut command = Task::none();
        match self {
            Self::CopyToClipBoard(input) => command = clipboard::write(input),
            Self::Notify(message) => app.appview.notification = Some(message),
        };
        command
    }
}
