use iced::{widget::{self, column}, Element, Length, Task};
use types::{crypto::Password, Notification};
use wallet::{wallet::Wallet, Setup};
use zeroize::Zeroize;

use crate::{common_elements, components::{self, password_input::password_input}, initial::common::{nav_button, nav_row}};


const VERIFY_PASSWORD_ID: u8 = 1;

#[derive(Clone)]
pub enum Message {
    Back,
    Next,
    SaveToWallet,
    SetNotification(Notification), 
    InputPassword(String),
    InputVerifyPassword(String),
    TogglePasswordVisibility,
    ToggleVerifyPasswordsVisibility,
    SetFocusVerifyPassword,
}

#[derive(Debug)]
pub struct EnterPassword {
    pub notification: Notification,
    pub password: Password,
    pub verify_password: Password,
    pub reveal_password: bool,
    pub reveal_verify_password: bool,
}

impl<'a> EnterPassword {
    pub fn new(password: Option<&str>, notification: Notification) -> Self {
        let password = match password {
            Some(password) => Password::from(password),
            None => Password::new(),
        };

        Self {
            notification,
            password: password.clone(),
            verify_password: password,
            reveal_password: false,
            reveal_verify_password: false,
        }
    }

    pub fn update(&mut self, message: Message, wallet: &'a mut Wallet<Setup>) -> Task<Message> {
        match message {
            Message::SetNotification(notification) => self.notification = notification,
            Message::InputPassword(input) => self.input_password(input),
            Message::InputVerifyPassword(input) => self.input_verify_password(input),
            Message::TogglePasswordVisibility => self.reveal_password =!self.reveal_password,
            Message::ToggleVerifyPasswordsVisibility => self.reveal_verify_password =!self.reveal_verify_password,
            Message::SetFocusVerifyPassword => {}
            Message::SaveToWallet => self.save_to_wallet(wallet),
            Message::Back | Message::Next => {/*Handle in parent*/}
        };
        Task::none()
    } 

    fn input_password(&mut self, mut input: String ) {
        self.password.replace(&input);
        input.zeroize();
        self.notification = Notification::None;
    }

    fn input_verify_password(&mut self, mut input: String) {
        self.verify_password.replace(&input);
        input.zeroize();
        self.notification = Notification::None;
    }

    pub fn save_to_wallet(&mut self, wallet: &'a mut Wallet<Setup>) {
        if self.password.as_str() == self.verify_password.as_str() {
            wallet.set_password(self.password.clone());
        } else {
            self.notification = Notification::Info("Passwords do not match".to_string());
        }
    }
}


impl<'a> EnterPassword {
    pub fn view(&'a self) -> Element<'a, Message> {
        let header = common_elements::header_one("Create password");

        let password_notification = components::notification::notification(&self.notification);

        let pw_input = password_input(
            "Enter Password",
            self.password.as_str(),    
            self.reveal_password, 
            Message::TogglePasswordVisibility,
            Message::InputPassword, 
            Message::SetFocusVerifyPassword
        );

        let verify_pw_input = password_input(
            "Verify Password",
            self.verify_password.as_str(),    
            self.reveal_verify_password, 
            Message::ToggleVerifyPasswordsVisibility,
            Message::InputVerifyPassword, 
            Message::SaveToWallet,
        );

        let content = widget::column![
            header,
            password_notification,
            pw_input,
            verify_pw_input,
        ]
        .align_x(iced::Alignment::Center)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .spacing(50);
        
        let nav = nav_row(
            nav_button("Back", Message::Back),
            nav_button("Next", Message::Next),
        );

        let content_and_nav = column![content, nav];

        widget::container(content_and_nav)
            .center_x(660)
            .center_y(700)
            .into()
    }
}