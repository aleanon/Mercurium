use bip39::Mnemonic;
use iced::{futures::executor::Enter, Task};
use types::crypto::{Password, SeedPhrase};
use zeroize::Zeroize;
use crate::app::{AppData, AppMessage};
use super::{set_password::SetPassword, Message, RestoreFromSeed, Stage, TaskResponse};
use iced::{
    widget::{self, text_input::Id, Column, TextInput},
    Length,
};
use zeroize::Zeroize;

use crate::{
    app::AppMessage,
    common_elements,
    initial::common::{nav_button, nav_row, seed_word_field},
};

use super::{Message, RestoreFromSeed};

pub struct EnterSeedPhrase {
    pub notification: &'static str,
    pub seed_phrase: SeedPhrase,
    pub seed_password: Option<Password>,
    pub mnemonic: Option<Mnemonic>,
}

impl Into<SetPassword> for EnterSeedPhrase {
    fn into(self) -> SetPassword {
        SetPassword {
            notification: "",
            mnemonic:  self.mnemonic.unwrap_or(Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English)),
            seed_password: self.seed_password,
            password: Password::new(),
            verify_password: Password::new(), 
        }
    }
}


impl<'a> EnterSeedPhrase {
    pub fn update_single_word_in_seed_phrase(&mut self, word_index: usize, mut word: String) {
        self.seed_phrase
            .update_word(word_index, word.as_str());
        word.zeroize();
        self.notification = "";
    }

    pub fn update_multiple_words_in_seed_phrase_from_index(&mut self, mut index: usize, words: Vec<String>) {
        for mut word in words {
            self.seed_phrase.update_word(index, &word);
            word.zeroize();
            index += 1;
        }
    }

    pub fn update_seed_password_field(&mut self, mut input: String) {
        self.seed_password.clear();
        self.seed_password.push_str(input.as_str());
        input.zeroize();
        self.seed_password
            .as_mut()
            .and_then(|password| Some(password.replace(input.as_str())));

        input.zeroize();
    }

    pub fn toggle_use_of_seed_password(&mut self) {
        if self.seed_password.is_none() {
            self.seed_password = Some(Password::new())
        } else {
            self.seed_password = None;
        }
    }

    pub fn goto_page_enter_password(&mut self, appdata: &'a mut AppData) -> Task<AppMessage> {
        let mnemonic = Mnemonic::from_phrase(
            self.seed_phrase.phrase().as_str(),
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


impl<'a> EnterSeedPhrase {
    pub fn view(&self) -> Column<'a, AppMessage> {
        let header = common_elements::header_one("Enter seed phrase");

        let notification = widget::text(self.notification).size(12);

        let input_seed = self.input_seed();

        let nav = nav_row(
            nav_button("Back").on_press(Message::Back.into()),
            nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![header, notification, input_seed, nav]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_x(iced::Alignment::Center)
            .spacing(50)
    }

    fn input_seed(&self) -> Column<'a, AppMessage> {
        let mut seed = widget::column![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);
        let mut row = widget::row![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);

        for index in 0..self.inputs.seed_phrase.nr_of_words() {
            if index % 4 == 0 && index != 0 {
                seed = seed.push(row);
                row = widget::row![]
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(20);
            }
            let word = self.inputs.seed_phrase.reference_word(index).unwrap_or("");

            let text_field = Self::seed_word_text_field_with_id(index, word);

            row = row.push(text_field);
        }
        seed.push(row)
    }

    fn seed_word_text_field_with_id(index: usize, word: &str) -> TextInput<'a, AppMessage> {
        seed_word_field(&format!("Word {}", index + 1), word)
            .id(Id::new(format!("{index}")))
            .on_input(move |input| Message::InputSeedWord((index, input)).into())
            .on_paste(move |mut string| {
                let input = string
                    .split_ascii_whitespace()
                    .map(|s| String::from(s))
                    .collect::<Vec<String>>();
                string.zeroize();
                Message::PasteSeedPhrase((index, input)).into()
            })
    }
}
