use iced::Task;
use types::crypto::{Password, PasswordError};
use zeroize::Zeroize;

use crate::{app::AppMessage, error::errorscreen::ErrorMessage};

use super::{Message, RestoreFromSeed, Stage, TaskResponse};
use iced::{
    widget::{self, Column},
    Length,
};

use crate::{
    app::AppMessage,
    common_elements,
    initial::common::{nav_button, nav_row, notification_field, text_input_field},
};

use super::{Message, RestoreFromSeed};


impl RestoreFromSeed {
    pub fn update_password(&mut self, mut input: String) {
        self.inputs.password.clear();
        self.inputs.password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }

    pub fn update_verify_password(&mut self, mut input: String) {
        self.inputs.verify_password.clear();
        self.inputs.verify_password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }

    pub fn goto_page_choose_account(&mut self) -> Task<AppMessage> {
        if self.inputs.password != self.inputs.verify_password {
            self.notification = "Passwords do not match";
            return Task::none();
        } else if self.inputs.password.len() < Password::MIN_LEN {
            self.notification = "Password must be at least 16 characters long";
            return Task::none();
        } else {
            self.notification = "";
            self.stage = Stage::ChooseAccounts;
            self.task_derive_encryption_keys_and_salt_for_mnemonic_and_database()
        }
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

impl<'a> RestoreFromSeed {
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
