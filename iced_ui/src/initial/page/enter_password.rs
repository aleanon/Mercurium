use iced::{widget::{self, column, container}, Element, Length, Task};
use types::{crypto::Password, Notification};
use wallet::{wallet::Wallet, Setup};
use zeroize::Zeroize;

use crate::{common_elements, components::{self, password_input::password_input}, initial::common::{self, nav_button, nav_row}};


const VERIFY_PASSWORD_FIELD_ID: u8 = 1;

#[derive(Clone)]
pub enum Message {
    Back,
    Next,
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
    pub fn new(wallet: &Wallet<wallet::Setup>, notification: Notification) -> Self {
        let password = match wallet.password() {
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

    pub fn input_is_valid(&mut self) -> bool {
        if self.password.len() < Password::MIN_LEN {
            self.notification = Notification::Info(format!("Password needs to be at least {} characters long", Password::MIN_LEN));
            return false
        }
        if self.password.as_str() != self.verify_password.as_str() {
            self.notification = Notification::Info("Passwords do not match".to_string());
            return false
        }
        true
    }

    pub fn save_to_wallet(&mut self, wallet: &'a mut Wallet<Setup>) -> Result<(), ()> {
        if self.password.len() < Password::MIN_LEN {
            self.notification = Notification::Info(format!("Password needs to be at least {} characters long", Password::MIN_LEN));
            return Err(())
        }
        if self.password.as_str() != self.verify_password.as_str() {
            self.notification = Notification::Info("Passwords do not match".to_string());
            return Err(())
        }
        wallet.set_password(self.password.clone());
        Ok(())
    }
}


impl<'a> EnterPassword {
    pub fn view(&'a self) -> Element<'a, Message> {
        // let password_notification = components::notification::notification(&self.notification);

        let header = common_elements::header_one("Create password");

        let notification = components::notification::notification(&self.notification);

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
            Message::Next,
        );

        let content = widget::column![
            header,
            notification,
            pw_input,
            verify_pw_input,
        ]
        .align_x(iced::Alignment::Center)
        .spacing(50);

        let content_container = container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        
        let nav = nav_row(
            nav_button("Back", Message::Back),
            nav_button("Next", Message::Next),
        );

        let content_and_nav = column![content_container, nav];

        widget::container(content_and_nav)
            .max_width(600)
            .max_height(550)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}