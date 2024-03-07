use iced::{futures::SinkExt, Command};
use types::Action;
use zeroize::Zeroize;

use crate::{app::{AppError, State}, App};

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
                if let State::Locked(ref mut loginscreen) = app.state {
                    loginscreen.password.clear();
                    loginscreen.password.push_str(string.as_str());
                    string.zeroize()
                }
            }
            LoginMessage::Login => {
                if let State::Locked(ref login) = app.state {
                    // let salt
                    let (key, _salt) = login.password.derive_new_db_encryption_key().unwrap();
                    //take the password, verify and create encryption key and decrypt the database

                    if let Err(err) = app.login() {
                        match err {
                            AppError::Fatal(_) => app.state = State::Error(err),
                            AppError::NonFatal(_) => { /*In app notification*/ }
                        }
                    };

                    // if let Err(err) = app.action_tx.send(Action::LoadDatabase(key)) {
                    //     app.state = State::Error(AppError::Fatal(Box::new(err)))
                    // }

                    if let Some(ref channel) = app.action_tx {
                        command = {
                            let mut connection = channel.clone();
                            Command::perform(
                                async move { connection.send(Action::LoadDatabase(key)).await },
                                |_| Message::None,
                            )
                        };
                    }
                }
            }
        }
        command
    }
}
