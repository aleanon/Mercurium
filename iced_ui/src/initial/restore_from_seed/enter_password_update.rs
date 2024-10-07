use iced::Task;
use types::crypto::{Password, PasswordError};
use zeroize::Zeroize;

use crate::{app::AppMessage, error::errorscreen::ErrorMessage};

use super::{Message, RestoreFromSeed, Stage, TaskResponse};

impl RestoreFromSeed {
    pub fn update_password_field(&mut self, mut input: String) {
        self.inputs.password.clear();
        self.inputs.password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }

    pub fn update_verify_password_field(&mut self, mut input: String) {
        self.inputs.verify_password.clear();
        self.inputs.verify_password.push_str(input.as_str());
        input.zeroize();
        self.notification = "";
    }

    pub fn from_enter_password_to_choose_account(&mut self) -> Task<AppMessage> {
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
