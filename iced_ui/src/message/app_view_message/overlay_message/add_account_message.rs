use iced::Command;
use zeroize::Zeroize;

use crate::{
    message::{app_view_message::AppViewMessage, Message},
    view::app_view::overlay::{
        add_account_view::{AddAccountView, View},
        Overlay,
    },
    App,
};

use super::OverlayMessage;

#[derive(Debug, Clone)]
pub enum AddAccountMessage {
    InputAccountName(String),
    InputPassword(String),
    Next,
}

impl Into<Message> for AddAccountMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::OverlayMessage(
            OverlayMessage::AddAccountMessage(self),
        ))
    }
}

impl AddAccountMessage {
    pub fn update(self, app: &mut App) -> Command<Message> {
        let command = Command::none();
        if let Some(Overlay::AddAccount(add_account_view)) = &mut app.appview.overlay {
            match self {
                Self::InputAccountName(input) => add_account_view.account_name = input,
                Self::InputPassword(input) => Self::update_password(input, add_account_view),
                Self::Next => Self::next(add_account_view),
            }
        }
        command
    }

    fn update_password(mut input: String, add_account_view: &mut AddAccountView) {
        add_account_view.password.clear();
        add_account_view.password.push_str(input.as_str());
        input.zeroize();
    }

    fn next(add_account_view: &mut AddAccountView) {
        match add_account_view.view {
            View::InputAccountName => add_account_view.view = View::InputPassword,
            View::InputPassword => {}
        };
    }
}
