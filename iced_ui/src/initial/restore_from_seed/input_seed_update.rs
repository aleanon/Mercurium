use bip39::Mnemonic;
use iced::Task;
use types::crypto::Password;
use zeroize::Zeroize;

use crate::app::{AppData, AppMessage};

use super::{Message, RestoreFromSeed, Stage, TaskResponse};

impl<'a> RestoreFromSeed {
    pub fn update_single_word_in_seed_phrase(&mut self, word_index: usize, mut word: String) {
        self.inputs
            .seed_phrase
            .update_word(word_index, word.as_str());
        word.zeroize();
        self.notification = "";
    }

    pub fn update_multiple_words_in_seed_phrase(&mut self, mut index: usize, words: Vec<String>) {
        for mut word in words {
            self.inputs.seed_phrase.update_word(index, &word);
            word.zeroize();
            index += 1;
        }
    }

    pub fn update_seed_password_field(&mut self, mut input: String) {
        self.inputs
            .seed_password
            .as_mut()
            .and_then(|password| Some(password.replace(input.as_str())));

        input.zeroize();
    }

    pub fn toggle_use_of_seed_password(&mut self) {
        if self.inputs.seed_password.is_none() {
            self.inputs.seed_password = Some(Password::new())
        } else {
            self.inputs.seed_password = None;
        }
    }

    pub fn goto_page_enter_password(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        let mnemonic = Mnemonic::from_phrase(
            self.inputs.seed_phrase.phrase().as_str(),
            bip39::Language::English,
        );
        let Ok(mnemonic) = mnemonic else {
            self.notification = "Invalid seed phrase";
            return Task::none();
        };

        self.mnemonic = Some(mnemonic.clone());
        self.stage = Stage::EnterPassword;
        self.notification = "";

        self.task_create_accounts_from_seed(appdata, mnemonic)
    }

    fn task_create_accounts_from_seed(
        &self,
        appdata: &'a mut AppData,
        mnemonic: Mnemonic,
    ) -> Task<AppMessage> {
        let password = self.inputs.seed_password.clone();
        let network = appdata.settings.network;
        let task_id = self.accounts_data.create_accounts_task_nr + 1;
        Task::perform(
            async move {
                let password_as_str = password
                    .as_ref()
                    .and_then(|password| Some(password.as_str()));

                let accounts = handles::wallet::create_multiple_accounts_from_mnemonic::<Vec<_>>(
                    &mnemonic,
                    password_as_str,
                    0,
                    0,
                    60,
                    network,
                );
                (task_id, accounts)
            },
            |(task_id, accounts)| {
                Message::TaskResponse(TaskResponse::AccountsCreated { task_id, accounts }).into()
            },
        )
    }
}
