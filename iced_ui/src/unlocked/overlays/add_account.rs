use std::collections::BTreeMap;

use handles::EncryptedMnemonic;
use iced::{
    futures::SinkExt,
    theme,
    widget::{self, button, column, text, text_input},
    Command, Element,
};
use ravault_iced_theme::styles;
use types::{crypto::Password, Action};
use zeroize::Zeroize;

use crate::{app::AppData, app::AppMessage, unlocked::app_view, CREDENTIALS_STORE_NAME};

use super::overlay;

#[derive(Debug, Clone)]
pub enum Message {
    InputAccountName(String),
    InputPassword(String),
    Next,
    Submit,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::OverlayMessage(
            overlay::Message::AddAccountMessage(self),
        ))
    }
}

#[derive(Debug, Clone)]
pub enum View {
    InputAccountName,
    InputPassword,
}

#[derive(Debug, Clone)]
pub struct AddAccountView {
    pub notification: String,
    pub account_name: String,
    pub password: Password,
    pub view: View,
}

impl AddAccountView {
    pub fn new() -> Self {
        Self {
            notification: String::new(),
            account_name: String::new(),
            password: Password::new(),
            view: View::InputAccountName,
        }
    }
}

impl<'a> AddAccountView {
    pub fn update(&mut self, message: Message, appdata: &mut AppData) -> Command<AppMessage> {
        let mut command = Command::none();

        match message {
            Message::InputAccountName(input) => self.update_account_name(input),
            Message::InputPassword(input) => self.update_password(input),
            Message::Next => self.next(),
            Message::Submit => command = self.submit(appdata),
        }

        command
    }

    fn update_account_name(&mut self, input: String) {
        if !input.is_empty() && !self.notification.is_empty() {
            self.notification.clear()
        }
        self.account_name = input;
    }

    fn update_password(&mut self, mut input: String) {
        self.password.clear();
        self.password.push_str(input.as_str());
        input.zeroize();
    }

    fn next(&mut self) {
        match self.view {
            View::InputAccountName => {
                if self.account_name.len() > 0 {
                    self.notification.clear();
                    self.view = View::InputPassword
                } else {
                    self.notification = "Account name cannot be empty".to_string();
                }
            }
            View::InputPassword => {}
        };
    }

    fn submit(&mut self, app_data: &mut AppData) -> Command<AppMessage> {
        let mut command = Command::none();
        let account =
            EncryptedMnemonic::from_store(CREDENTIALS_STORE_NAME).and_then(|encrypted_mnemonic| {
                encrypted_mnemonic
                    .decrypt_mnemonic(&self.password)
                    .and_then(|mnemonic| {
                        let accounts = app_data.db.get_accounts().unwrap_or(BTreeMap::new());
                        let mut id = 0;
                        let mut new_account_index = 0;

                        for (_, account) in accounts {
                            if account.id >= id {
                                id = account.id + 1
                            };
                            let account_index = account.derivation_index();
                            if account_index >= new_account_index {
                                new_account_index = account_index + 1
                            };
                        }
                        Ok(handles::wallet::create_account_from_mnemonic(
                            &mnemonic,
                            id,
                            new_account_index,
                            self.account_name.clone(),
                            app_data.settings.network,
                        ))
                    })
            });
        match account {
            Ok(account) => match app_data.db.upsert_account(&account) {
                Ok(()) => {
                    let mut sender = app_data.backend_sender.clone();
                    command = Command::perform(
                        async move { sender.send(Action::UpdateAccount(account.address)).await },
                        |_| AppMessage::None,
                    )
                }
                Err(err) => {
                    self.notification = format!("Unable to save account to database: {err}");
                }
            },
            Err(err) => self.notification = format!("Unable to create account: {err}"),
        };

        command
    }

    pub fn view(&self, appdata: &'a AppData) -> Element<'a, AppMessage> {
        let content = match self.view {
            View::InputAccountName => self.input_account_name(),
            View::InputPassword => self.input_password(),
        };
        let notification = text(&self.notification);

        let column = column![notification, content];

        widget::container(column)
            .width(400)
            .height(400)
            .padding(10)
            .center_x()
            .center_y()
            .style(styles::container::OverlayInner::style)
            .into()
    }

    fn input_account_name(&self) -> Element<'a, AppMessage> {
        let account_name_input = {
            let label = text("Account name");
            let account_name_input = text_input("Enter account name", &self.account_name)
                .style(theme::TextInput::Custom(Box::new(
                    styles::text_input::GeneralInput,
                )))
                .on_input(|input| Message::InputAccountName(input).into());

            column!(label, account_name_input).spacing(2)
        };

        let next_button = button("Next").on_press(Message::Next.into());

        column![account_name_input, next_button].spacing(20).into()
    }

    fn input_password(&self) -> Element<'a, AppMessage> {
        let password_input = {
            let label = text("Password");
            let password_input = text_input("Enter password", &self.password.as_str())
                .style(theme::TextInput::Custom(Box::new(
                    styles::text_input::GeneralInput,
                )))
                .on_input(|input| Message::InputPassword(input).into());

            column![label, password_input].spacing(2)
        };

        let submit_button = button("Submit").on_press(Message::Submit.into());

        column![password_input, submit_button].spacing(20).into()
    }
}
