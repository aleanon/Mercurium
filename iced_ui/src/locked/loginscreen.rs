use std::mem;

use debug_print::debug_println;
use font_and_icons::images::MENU_LOGO;
use iced::{
    widget::{self, image::Handle},
    Element, Length, Task,
};
use types::{crypto::Password, AppError};
use zeroize::Zeroize;

use crate::{
    app::{AppData, AppMessage}, components::password_input::password_input, external_task_response, external_tasks
};

#[derive(Debug, Clone)]
pub enum Message {
    Login,
    LoginFailed(String),
    LoginSuccess,
    PasswordInput(String),
    ToggleShowPassword,
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

    pub fn update(&mut self, message: Message, appdata: &'a mut AppData) -> Task<AppMessage> {
        match message {
            Message::Login => {
                self.status = Status::LoggingIn;
                return self.login(appdata);
            }
            Message::LoginFailed(info) => {
                self.status = Status::Input;
                self.notification = info;
            }
            Message::LoginSuccess => {
                if self.application_is_starting {
                    return external_tasks::initial_login_tasks(appdata.settings.network);
                };
            }
            Message::PasswordInput(input) => self.input(input),
            Message::ToggleShowPassword => self.toggle_view(),
        }
        Task::none()
    }

    fn login(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        debug_println!("Logging in");

        self.status = Status::LoggingIn;
        let password = mem::take(&mut self.password);
        let network = appdata.settings.network;
        Task::perform(
                async move { handles::wallet::perform_login_check(network, &password).await },
            |result| match result {
                Ok(_) => Message::LoginSuccess.into(),
                Err(err) =>{
                    debug_println!("failed login check: {err}");
                    match err {
                        AppError::Fatal(_) => external_task_response::Message::Error(err).into(),
                        AppError::NonFatal(notification) => {
                            Message::LoginFailed(notification.to_string()).into()
                        }
                    }
                }
            },
        )
    }

    pub fn view(&self) -> Element<'a, Message> {
        // if self.status == Status::LoggingIn {
        //     return
        // }

        let logo = widget::image(Handle::from_bytes(MENU_LOGO)).width(100).height(100);

        let info_text  = widget::text("Enter password to continue")
            .size(15);

        let space = widget::vertical_space().height(15);

        let password_input = password_input(
            self.password.as_str(),
            self.show_password,
            Message::ToggleShowPassword,
            Message::PasswordInput,
            Message::PasswordInput,
            Message::Login)
                .width(200)
                .height(30);

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
