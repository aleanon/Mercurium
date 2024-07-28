use iced::{
    widget::{self, button, column, row, text, text_input, Space},
    Element, Length, Task,
};
use ravault_iced_theme::styles;
use std::sync::Arc;
use types::crypto::Password;
use zeroize::Zeroize;

use crate::{
    app::{AppData, AppMessage},
    external_task_response,
    unlocked::app_view,
};

pub static INPUT_ACCOUNT_NAME: &'static str = "input_account_name";
pub static INPUT_PASSWORD: &'static str = "input_password";

use super::overlay;

#[derive(Debug, Clone)]
pub enum Message {
    InputAccountName(String),
    InputPassword(String),
    Back,
    Continue,
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

impl<'a> AddAccountView {
    pub fn new() -> (Self, Task<AppMessage>) {
        let add_account_view = Self {
            notification: String::new(),
            account_name: String::new(),
            password: Password::new(),
            view: View::InputAccountName,
        };

        let task = text_input::focus(text_input::Id::new(INPUT_ACCOUNT_NAME));

        (add_account_view, task)
    }

    pub fn update(&mut self, message: Message, appdata: &mut AppData) -> Task<AppMessage> {
        match message {
            Message::InputAccountName(input) => self.update_account_name(input),
            Message::InputPassword(input) => self.update_password(input),
            Message::Back => return self.back(),
            Message::Continue => return self.next(),
            Message::Submit => return self.submit(appdata),
        }
        Task::none()
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

    fn back(&mut self) -> Task<AppMessage> {
        if let View::InputPassword = self.view {
            self.view = View::InputAccountName;
            return text_input::focus(text_input::Id::new(INPUT_ACCOUNT_NAME));
        }
        Task::none()
    }

    fn next(&mut self) -> Task<AppMessage> {
        match self.view {
            View::InputAccountName => {
                if self.account_name.len() > 0 {
                    self.notification.clear();
                    self.view = View::InputPassword;
                    return text_input::focus(text_input::Id::new(INPUT_PASSWORD));
                } else {
                    self.notification = "Account name cannot be empty".to_string();
                }
            }
            View::InputPassword => {}
        };
        Task::none()
    }

    fn submit(&mut self, app_data: &mut AppData) -> Task<AppMessage> {
        let mut task = Task::none();
        let account =
            handles::credentials::get_encrypted_mnemonic().and_then(|encrypted_mnemonic| {
                encrypted_mnemonic
                    .decrypt_mnemonic(&self.password)
                    .and_then(|mnemonic| {
                        let mut id = 0;
                        let mut new_account_index = 0;

                        for (_, account) in app_data.accounts.iter() {
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
                            None,
                            id,
                            new_account_index,
                            self.account_name.clone(),
                            app_data.settings.network,
                        ))
                    })
                    .map_err(|err| {
                        types::AppError::NonFatal(types::Notification::Warn(err.to_string()))
                    })
            });
        match account {
            Ok(account) => {
                app_data
                    .accounts
                    .insert(account.address.clone(), account.clone());

                let network = app_data.settings.network;
                let resources = app_data.resources.clone();
                task = Task::perform(
                    async move {
                        let accounts_update = handles::radix_dlt::updates::update_accounts(
                            network,
                            Arc::new(resources),
                            vec![account],
                        )
                        .await;
                        Ok(accounts_update)
                    },
                    |result| match result {
                        Ok(accounts_update) => {
                            external_task_response::Message::AccountsUpdated(accounts_update).into()
                        }
                        Err(err) => external_task_response::Message::Error(err).into(),
                    },
                )
            }
            Err(err) => self.notification = format!("Unable to create account: {err}"),
        };

        task
    }

    pub fn view(&'a self, appdata: &'a AppData) -> Element<'a, AppMessage> {
        let content = match self.view {
            View::InputAccountName => self.input_account_name(),
            View::InputPassword => self.input_password(),
        };
        let notification = text(&self.notification);

        let column = column![notification, content];

        widget::container(column)
            .padding(10)
            .center_x(400)
            .center_y(400)
            .style(styles::container::overlay_inner)
            .into()
    }

    fn input_account_name(&self) -> Element<'a, AppMessage> {
        let header = text("Create new account")
            .size(16)
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .vertical_alignment(iced::alignment::Vertical::Center);

        let top_space = Space::with_height(Length::Fill);

        let account_name_input = {
            let label = text("Account name");
            let account_name_input = text_input("Enter account name", &self.account_name)
                .style(styles::text_input::general_input)
                .on_submit(Message::Continue.into())
                .on_input(|input| Message::InputAccountName(input).into())
                .id(text_input::Id::new(INPUT_ACCOUNT_NAME));

            column!(label, account_name_input).spacing(2)
        };

        let bottom_space = Space::with_height(Length::Fill);
        let continue_button = button("continue").on_press_maybe(if !self.password.is_empty() {
            Some(Message::Continue.into())
        } else {
            None
        });

        column![
            header,
            top_space,
            account_name_input,
            bottom_space,
            continue_button
        ]
        .align_items(iced::Alignment::Center)
        .spacing(20)
        .into()
    }

    fn input_password(&self) -> Element<'a, AppMessage> {
        let password_input = {
            let label = text("Password");
            let password_input = text_input("Enter password", &self.password.as_str())
                .style(styles::text_input::general_input)
                .on_input(|input| Message::InputPassword(input).into())
                .on_submit(Message::Submit.into())
                .id(text_input::Id::new(INPUT_PASSWORD))
                .secure(true);

            column![label, password_input].spacing(2)
        };

        let space = Space::with_height(Length::Fill);
        let back_button = button("Back").on_press(Message::Back.into());
        let submit_button = button("Submit").on_press_maybe(if self.password.is_empty() {
            None
        } else {
            Some(Message::Submit.into())
        });

        let buttons_row = row!(
            Space::with_width(Length::Fill),
            back_button,
            submit_button,
            Space::with_width(Length::Fill)
        )
        .spacing(30);

        column![password_input, space, buttons_row]
            .align_items(iced::Alignment::Center)
            .spacing(20)
            .into()
    }
}
