use iced::Command;

use crate::{
    app::{AppMessage, AppState},
    App,
};

#[derive(Debug, Clone)]
pub enum ErrorMessage {
    Notify(String),
    Fatal(String),
    Ignore(String),
}

impl Into<AppMessage> for ErrorMessage {
    fn into(self) -> AppMessage {
        AppMessage::Error(self)
    }
}

impl<'a> ErrorMessage {
    pub fn update(self, app: &'a mut App) -> Command<AppMessage> {
        match self {
            Self::Notify(error) => Self::notify(error, app),
            Self::Fatal(error) => Self::fatal(error, app),
            Self::Ignore(error) => Command::none(),
        }
    }

    fn notify(error: String, app: &'a mut App) -> Command<AppMessage> {
        app.appview.notification = Some(error);
        Command::none()
    }

    fn fatal(error: String, app: &'a mut App) -> Command<AppMessage> {
        app.app_state = AppState::Error(error);
        Command::none()
    }
}
