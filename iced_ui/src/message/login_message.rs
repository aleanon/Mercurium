use iced::{futures::SinkExt, Command};
use types::{Action, AppError};
use zeroize::Zeroize;

use crate::{app::AppState, App};

use super::Message;

#[derive(Debug, Clone)]
pub enum LoginMessage {
  TextInputChanged(String),
  Login,
}

impl Into<Message> for LoginMessage {
    fn into(self) -> Message {
        Message::Login(self)
    }
}


impl<'a> LoginMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();

        match self {
            LoginMessage::TextInputChanged(mut string) => {
                if let AppState::Locked(ref mut loginscreen) = app.app_state {
                    loginscreen.password.clear();
                    loginscreen.password.push_str(string.as_str());
                    string.zeroize()
                }
            }
            LoginMessage::Login => {
                if let AppState::Locked(ref login) = app.app_state {
                    // let salt
                    let (key, _salt) = login.password.derive_new_db_encryption_key().unwrap();
                    //take the password, verify and create encryption key and decrypt the database

                    if let Err(err) = app.login() {
                        match err {
                            AppError::Fatal(_) => app.app_state = AppState::Error(err),
                            AppError::NonFatal(_) => { /*In app notification*/ }
                        }
                    };

                    // if let Err(err) = app.action_tx.send(Action::LoadDatabase(key)) {
                    //     app.state = State::Error(AppError::Fatal(Box::new(err)))
                    // }

                    command = {
                        let mut connection = app.app_data.backend_sender.clone();
                        Command::perform(
                            async move { connection.send(Action::LoadDatabase(key)).await },
                            |_| Message::None,
                        )
                    };
                    
                }
            }
        }
        command
    }
}
