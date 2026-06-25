use std::iter::Take;

use deps::{
    iced::{advanced::widget::operation::focusable::focus, widget::Id},
    *,
};

use iced::{
    Element, Length, Task,
    widget::{self, Space, button, column, row, text, text_input},
};
use types::crypto::Password;
use wallet::{Unlocked, Wallet};
use zeroize::Zeroize;

use crate::{app::AppMessage, styles, unlocked::app_view};

pub const INPUT_ACCOUNT_NAME: &'static str = "input_account_name";
pub const INPUT_PASSWORD: &'static str = "input_password";

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
pub struct AddAccount {
    pub notification: String,
    pub account_name: String,
    pub password: Password,
    pub view: View,
}

impl<'a> AddAccount {
    pub fn new() -> (Self, Task<AppMessage>) {
        let add_account_view = Self {
            notification: String::new(),
            account_name: String::new(),
            password: Password::new(),
            view: View::InputAccountName,
        };

        // let task = focus(Id::from(INPUT_ACCOUNT_NAME));

        (add_account_view, Task::none())
    }

    pub fn update(&mut self, message: Message, wallet: &mut Wallet<Unlocked>) -> Task<AppMessage> {
        match message {
            Message::InputAccountName(input) => self.update_account_name(input),
            Message::InputPassword(input) => self.update_password(input),
            Message::Back => return self.back(),
            Message::Continue => return self.next(),
            Message::Submit => return self.submit(wallet),
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
        }
        Task::none()
    }

    fn next(&mut self) -> Task<AppMessage> {
        match self.view {
            View::InputAccountName => {
                if self.account_name.len() > 0 {
                    self.notification.clear();
                    self.view = View::InputPassword;
                } else {
                    self.notification = "Account name cannot be empty".to_string();
                }
            }
            View::InputPassword => {}
        };
        Task::none()
    }

    fn submit(&mut self, wallet: &mut Wallet<Unlocked>) -> Task<AppMessage> {
        // let Ok(join_handle) = wallet.create_new_account(account_name, password) else {
        //     return Task::none();
        // };
        // Task::perform(async move { join_handle.await }, |result| match result {
        //     Ok(account) => )}
        // })
        Task::none()
    }

    pub fn view(&'a self) -> Element<'a, AppMessage> {
        let content = match self.view {
            View::InputAccountName => self.input_account_name(),
            View::InputPassword => self.input_password(),
        };
        // let notification = text(&self.notification);

        // let column = column![notification, content];

        widget::container(content)
            .padding(10)
            .center_x(400)
            .center_y(400)
            .style(styles::container::overlay_inner)
            .into()
    }

    fn input_account_name(&'a self) -> Element<'a, AppMessage> {
        let header = text("Create new account")
            .size(16)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        let top_space = widget::space::vertical();

        let account_name_input = {
            let label = text("Account name");
            let account_name_input = text_input("Enter account name", &self.account_name)
                .style(styles::text_input::general_input)
                .on_submit(Message::Continue.into())
                .on_input(|input| Message::InputAccountName(input).into())
                .id(Id::from(INPUT_ACCOUNT_NAME))
                .padding(10);

            let notification = text(&self.notification).size(11);

            column!(label, account_name_input, notification).spacing(10)
        };

        let bottom_space = widget::space::vertical();
        let continue_button = button("continue").on_press_maybe(if !self.account_name.is_empty() {
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
        .align_x(iced::Alignment::Center)
        .spacing(20)
        .into()
    }

    fn input_password(&'a self) -> Element<'a, AppMessage> {
        let password_input = {
            let label = text("Password");
            let password_input = text_input("Enter password", &self.password.as_str())
                .style(styles::text_input::general_input)
                .on_input(|input| Message::InputPassword(input).into())
                .on_submit(Message::Submit.into())
                .id(Id::from(INPUT_PASSWORD))
                .secure(true)
                .padding(10);

            let notification = text(&self.notification);

            column![label, password_input, notification].spacing(10)
        };

        let space = widget::space::vertical();
        let back_button = button("Back").on_press(Message::Back.into());
        let submit_button = button("Submit").on_press_maybe(if self.password.is_empty() {
            None
        } else {
            Some(Message::Submit.into())
        });

        let buttons_row = row!(
            widget::space::horizontal(),
            back_button,
            submit_button,
            widget::space::horizontal(),
        )
        .spacing(30);

        column![password_input, space, buttons_row]
            .align_x(iced::Alignment::Center)
            .spacing(20)
            .into()
    }
}
