use debug_print::debug_println;
use iced::{
    widget::{self, text::LineHeight, text_input::Id},
    Element, Length, Task,
};
use store::AsyncDb;
use types::{crypto::Password, AppError};
use zeroize::Zeroize;

use crate::{
    app::{AppData, AppMessage},
    external_task_response, external_tasks,
};

#[derive(Debug, Clone)]
pub enum Message {
    TextInputChanged(String),
    Login,
    LoginFailed(String),
    LoginSuccess,
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
}

impl<'a> LoginScreen {
    pub fn new(on_application_statup: bool) -> Self {
        Self {
            status: Status::Input,
            application_is_starting: on_application_statup,
            notification: String::new(),
            password: Password::new(),
        }
    }

    pub fn password(&self) -> &Password {
        &self.password
    }

    pub fn update(&mut self, message: Message, appdata: &'a mut AppData) -> Task<AppMessage> {
        match message {
            Message::TextInputChanged(mut string) => {
                self.password.clear();
                self.password.push_str(string.as_str());
                string.zeroize()
            }
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
        }
        Task::none()
    }

    fn login(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        self.status = Status::LoggingIn;
        let password = self.password.clone();
        self.password.clear();
        let network = appdata.settings.network;
        Task::perform(
            async move { handles::wallet::perform_login_check(network, password).await },
            |result| match result {
                Ok(_) => Message::LoginSuccess.into(),
                Err(err) => match err {
                    AppError::Fatal(_) => external_task_response::Message::Error(err).into(),
                    AppError::NonFatal(notification) => {
                        Message::LoginFailed(notification.to_string()).into()
                    }
                },
            },
        )
    }

    pub fn view(&self) -> Element<'a, AppMessage> {
        let text_field = widget::text_input("Enter Password", &self.password.as_str())
            .secure(true)
            .width(250)
            .line_height(LineHeight::Relative(2.))
            .on_submit(Message::Login.into())
            .size(15)
            .id(Id::new("password_input"))
            .on_input(|value| Message::TextInputChanged(value).into());

        let login_button = widget::Button::new(
            widget::text("Login")
                .size(15)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
        .height(30)
        .width(100)
        .style(widget::button::primary)
        .on_press(Message::Login.into());

        let col = widget::column![text_field, login_button]
            .height(Length::Shrink)
            .width(Length::Shrink)
            .align_items(iced::Alignment::Center)
            .spacing(30);

        widget::container(col)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
