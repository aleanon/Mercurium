use iced::Command;

use crate::{
    message::{app_view_message::AppViewMessage, Message},
    view::app_view::overlay::{receive::Notification, Overlay},
    App,
};

use super::OverlayMessage;

#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    CopyAddress(String),
}

impl Into<Message> for ReceiveMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::OverlayMessage(
            OverlayMessage::ReceiveMessage(self),
        ))
    }
}

impl<'a> ReceiveMessage {
    pub fn update(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();
        if let Some(Overlay::Receive(receive_view)) = &mut app.appview.overlay {
            match self {
                Self::CopyAddress(address) => {
                    receive_view.notification =
                        Notification::Success("Address copied to clipboard".to_string());
                    command = iced::clipboard::write(address)
                }
            }
        }
        command
    }
}
