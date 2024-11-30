
use bip39::Mnemonic;
use iced::{Element, Task};
use types::{crypto::Password, AppError};
use zeroize::Zeroize;

use crate::{app::AppMessage, initial::{restore_from_seed, setup}};

use super::choose_account::ChooseAccounts;
use iced::{
    widget,
    Length,
};

use crate::{
    common_elements,
    initial::common::{notification_field, text_input_field},
};

#[derive(Debug, Clone)]
pub enum Message {
    InputPassword(String),
    InputVerifyPassword(String),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(
            setup::Message::RestoreFromSeedMessage(
                restore_from_seed::Message::SetPasswordMessage(self)
            )
        )
    }
}

#[derive(Debug)]
pub struct SetPassword {
    pub notification: &'static str,
    pub mnemonic: Mnemonic,
    pub seed_password: Option<Password>,
    pub password: Password,
    pub reveal_password: bool,
    pub verify_password: Password,
    pub reveal_verify_password: bool,
}


impl SetPassword {
    pub fn new() -> Self {
        Self {
            notification: "",
            mnemonic: Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English),
            seed_password: None,
            password: Password::new(),
            reveal_password: false,
            verify_password: Password::new(),
            reveal_verify_password: false,
        }
    }

    pub fn with_mnemonic_and_password(mnemonic: Mnemonic, seed_password: Option<Password>) -> Self {
        Self {
            notification: "",
            mnemonic,
            seed_password: seed_password,
            password: Password::new(),
            reveal_password: false,
            verify_password: Password::new(),
            reveal_verify_password: false,
        }
    }

    pub fn from_page_choose_account(choose_account: ChooseAccounts) -> Self {
        Self {
            notification: "",
            mnemonic: choose_account.mnemonic,
            seed_password: choose_account.seed_password,
            password: choose_account.password.clone(),
            reveal_password: false,
            verify_password: choose_account.password,
            reveal_verify_password: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Result<Task<AppMessage>, AppError> {
        match message {
            Message::InputPassword(input) => self.input_password(input),
            Message::InputVerifyPassword(input) => self.input_verify_password(input),
        }

        Ok(Task::none())
    }

    pub fn input_password(&mut self, mut input: String) {
        self.password.clear();
        self.password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }

    pub fn input_verify_password(&mut self, mut input: String) {
        self.verify_password.clear();
        self.verify_password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }

}

impl<'a> SetPassword {
    pub fn view(&self) -> Element<'a, AppMessage> {
        let header = common_elements::header_one("Create password");

        let password_notification = notification_field(self.notification);

        let password_input = text_input_field("Enter Password", &self.password.as_str())
            .on_paste(|input| Message::InputPassword(input).into())
            .on_input(|input| Message::InputPassword(input).into())
            .secure(!self.reveal_password);

        let verify_pw_input =
            text_input_field("Verify Password", &self.verify_password.as_str())
                .on_submit(restore_from_seed::Message::Next.into())
                .on_paste(|input| Message::InputVerifyPassword(input).into())
                .on_input(|input| Message::InputVerifyPassword(input).into())
                .secure(!self.reveal_verify_password);

        widget::column![
            header,
            password_notification,
            password_input,
            verify_pw_input,
        ]
        .align_x(iced::Alignment::Center)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .spacing(50)
        .into()
    }
}
