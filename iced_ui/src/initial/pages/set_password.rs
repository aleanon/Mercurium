
use bip39::Mnemonic;
use iced::Task;
use types::crypto::{Password, PasswordError, SeedPhrase};
use zeroize::Zeroize;

use crate::{app::AppMessage, error::errorscreen::ErrorMessage};

use super::enter_seedphrase::EnterSeedPhrase;
use iced::{
    widget::{self, Column},
    Length,
};

use crate::{
    common_elements,
    initial::common::{nav_button, nav_row, notification_field, text_input_field},
};

#[derive(Debug, Clone)]
pub enum Message {
    InputPassword(String),
    InputVerifyPassword(String),
}

#[derive(Debug)]
pub struct SetPassword {
    pub notification: &'static str,
    pub mnemonic: Mnemonic,
    pub seed_password: Option<Password>,
    pub password: Password,
    pub verify_password: Password,
}


impl SetPassword {
    pub fn from_page_enter_seedphrase(enter_seedphrase: EnterSeedPhrase) -> Result<Self,&str> {
        let phrase = enter_seedphrase.seed_phrase.phrase();
        let mnemonic = Mnemonic::from_phrase(phrase, bip39::Language::English).map_err(|_| "Invalid seed phrase")?;
        Ok(Self {
            notification: "",
            mnemonic,
            seed_password: enter_seedphrase.seed_password,
            password: Password::new(),
            verify_password: Password::new(),
        })
    }

    pub fn update(&mut self, message: Message) -> Task<AppMessage> {
        match message {
            Message::InputPassword(input) => self.update_password(input),
            Message::InputVerifyPassword(input) => self.update_verify_password(input),
        }
    }

    pub fn update_password(&mut self, mut input: String) {
        self.password.clear();
        self.password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }

    pub fn update_verify_password(&mut self, mut input: String) {
        self.verify_password.clear();
        self.verify_password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }


    fn task_derive_encryption_keys_and_salt_for_mnemonic_and_database(
        &mut self,
    ) -> Task<AppMessage> {
        let password = self.inputs.password.clone();
        let task_id = self.key_and_salt.last_task_nr + 1;
        Task::perform(
            async move {
                let db_key_salt = password.derive_new_db_encryption_key()?;
                let mnemonic_key_salt = password.derive_new_mnemonic_encryption_key()?;

                Ok::<_, PasswordError>((task_id, db_key_salt, mnemonic_key_salt))
            },
            |result| match result {
                Ok((task_id, db_key_salt, mnemonic_key_salt)) => {
                    Message::TaskResponse(TaskResponse::DbAndMnemonicKeySaltReceived {
                        task_id,
                        db_key_salt,
                        mnemonic_key_salt,
                    })
                    .into()
                }
                Err(err) => AppMessage::Error(ErrorMessage::Fatal(err.to_string())),
            },
        )
    }
}

impl<'a> SetPassword {
    pub fn enter_password_view(&self) -> Column<'a, AppMessage> {
        let header = common_elements::header_one("Create password");

        let password_notification = notification_field(self.notification);

        let password_input = text_input_field("Enter Password", &self.inputs.password.as_str())
            .on_paste(|input| Message::InputPassword(input).into())
            .on_input(|input| Message::InputPassword(input).into())
            .secure(true);

        let verify_pw_input =
            text_input_field("Verify Password", &self.inputs.verify_password.as_str())
                .on_submit(Message::Next.into())
                .on_paste(|input| Message::InputVerifyPassword(input).into())
                .on_input(|input| Message::InputVerifyPassword(input).into())
                .secure(true);

        let nav = nav_row(
            nav_button("Back").on_press(Message::Back.into()),
            nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![
            header,
            password_notification,
            password_input,
            verify_pw_input,
            nav
        ]
        .align_x(iced::Alignment::Center)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .spacing(50)
    }
}
