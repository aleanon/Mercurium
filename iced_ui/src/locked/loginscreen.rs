use deps::*;

use std::mem;

use debug_print::debug_println;
use font_and_icons::images::MENU_LOGO;
use iced::{
    widget::{self, image::Handle},
    Element, Length, Task,
};
use types::{crypto::Password, debug_info};
use wallet::{Locked, LoginResponse, Wallet};
use zeroize::Zeroize;

use crate::{
    app::AppMessage, components::password_input::password_input
};

#[derive(Clone)]
pub enum Message {
    Login,
    PasswordInput(String),
    ToggleShowPassword,
    LoginFailed(Wallet<Locked>, String),
    LoginSuccess(Wallet<wallet::Unlocked>, bool),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Login(self)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Input,
    LoggingIn,
}

#[derive(Debug, Clone)]
pub struct LoginScreen {
    pub status: Status,
    pub application_is_starting: bool,
    pub notification: String,
    pub password: Password,
    pub show_password: bool,
}

impl<'a> LoginScreen {
    pub fn new(on_application_statup: bool) -> Self {
        Self {
            status: Status::Input,
            application_is_starting: on_application_statup,
            notification: String::new(),
            password: Password::new(),
            show_password: false,
        }
    }

    fn toggle_view(&mut self) {
        self.show_password = !self.show_password;
    }

    fn input(&mut self, mut input: String) {
        self.password.clear();
        self.password.push_str(&input);
        input.zeroize();
    }

    pub fn update(&mut self, message: Message, wallet: &'a mut Wallet<Locked>) -> Task<Message> {
        match message {
            Message::Login => {
                self.status = Status::LoggingIn;
                return self.login(wallet);
            }
            Message::LoginFailed(wallet_locked, info) => {
                self.status = Status::Input;
                *wallet = wallet_locked;
                self.notification = info;
            }
            Message::PasswordInput(input) => self.input(input),
            Message::ToggleShowPassword => self.toggle_view(),
            Message::LoginSuccess(_,_) => {/*Propagated*/}
        }
        Task::none()
    }

    fn login(&mut self, wallet: &'a mut Wallet<Locked>) -> Task<Message> {
        debug_println!("Logging in");

        self.status = Status::LoggingIn;
        let wallet = mem::take(wallet);
        let password = mem::take(&mut self.password);

        Task::perform(
                async move { wallet.login_with_password(password).await },
            |response| match response {
                LoginResponse::Success(wallet, is_initial_login) => {
                    debug_println!("Login successful");
                    Message::LoginSuccess(wallet, is_initial_login)
                }
                LoginResponse::Failed(wallet, error) => {
                    debug_println!("Login Failed");
                    Message::LoginFailed(wallet, error.to_string())
                }
            },
        )
    }

    #[inline_tweak::tweak_fn]
    pub fn view(&self) -> Element<'a, Message> {
        // if self.status == Status::LoggingIn {
        //     return
        // }

        let logo = widget::image(Handle::from_bytes(MENU_LOGO)).width(100).height(100);

        let info_text  = widget::text("Enter password to continue")
            .size(15);

        let space = widget::vertical_space().height(15);

        let password_input = password_input(
            "Enter Password",
            self.password.as_str(),
            self.show_password,
            Message::ToggleShowPassword,
            Message::PasswordInput,
            Message::Login
        );

        let login_button = widget::Button::new(
            widget::text("Login")
                .size(15)
                .center()
                .width(Length::Fill)
                .height(Length::Fill)
        )
        .height(30)
        .width(100)
        .style(widget::button::primary)
        .on_press(Message::Login);

        let col = widget::column![logo, space, info_text, password_input, login_button]
            .height(Length::Shrink)
            .width(Length::Shrink)
            .align_x(iced::Alignment::Center)
            .spacing(30);

        widget::container(col)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
