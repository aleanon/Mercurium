use bip39::Mnemonic;
use iced::{Element, Task};
use types::{crypto::{Password, SeedPhrase}, AppError};
use zeroize::Zeroize;
use crate::{app::{AppData, AppMessage}, initial::{restore_from_seed, setup}};
use iced::{
    widget::{self, text_input::Id, Column, TextInput},
    Length,
};

use crate::{
    common_elements,
    initial::common::{nav_button, nav_row, seed_word_field},
};

use super::set_password::SetPassword;

#[derive(Debug, Clone)]
pub enum Message {
    UpdateSingleWordInSeedPhrase(usize, String),
    UpdateMultipleWordsInSeedPhraseFromIndex(usize, String),
    ToggleSeedPassword,
    InputSeedPassword(String),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(
            setup::Message::RestoreFromSeedMessage(
                restore_from_seed::Message::EnterSeedPhraseMessage(self)
            )
        )
    }
}

#[derive(Debug)]
pub struct EnterSeedPhrase {
    pub notification: &'static str,
    pub seed_phrase: SeedPhrase,
    pub seed_password: Option<Password>,
    pub mnemonic: Option<Mnemonic>,
}

impl EnterSeedPhrase {
    pub fn new() -> Self {
        Self {
            notification: "",
            seed_phrase: SeedPhrase::new(),
            seed_password: None,
            mnemonic: None,
        }
    }

    pub fn from_page_set_password(page: SetPassword) -> Self {
        Self {
            notification: "",
            seed_phrase: SeedPhrase::from_str(page.mnemonic.phrase()),
            seed_password: page.seed_password,
            mnemonic: None,
        }
    }
}

// Update
impl<'a> EnterSeedPhrase {
    pub fn update(&mut self, message: Message) -> Result<Task<AppMessage>, AppError> {
        match message {
            Message::UpdateSingleWordInSeedPhrase(word_index, word) => self.update_single_word_in_seed_phrase(word_index, word),
            Message::UpdateMultipleWordsInSeedPhraseFromIndex(word_index, words ) => self.update_multiple_words_in_seed_phrase_from_index(word_index, words),
            Message::InputSeedPassword(input) => self.update_seed_password_field(input),
            Message::ToggleSeedPassword => self.toggle_use_of_seed_password(),
        }

        Ok(Task::none())
    }

    pub fn update_single_word_in_seed_phrase(&mut self, word_index: usize, mut word: String) {
        self.seed_phrase
            .update_word(word_index, word.as_str());
        word.zeroize();
        self.notification = "";
    }

    pub fn update_multiple_words_in_seed_phrase_from_index(&mut self, mut word_index: usize, mut input: String) {
        let mut words = input.split_ascii_whitespace();

        while let Some(word) = words.next() && word_index < self.seed_phrase.nr_of_words() {
            self.seed_phrase.update_word(word_index, &word);
            word_index += 1;
        }
        input.zeroize();
    }

    pub fn update_seed_password_field(&mut self, mut input: String) {
        if let Some(password) = &mut self.seed_password {
            password.clear();
            password.push_str(input.as_str());
        }
        input.zeroize();
    }

    pub fn toggle_use_of_seed_password(&mut self) {
        if self.seed_password.is_none() {
            self.seed_password = Some(Password::new())
        } else {
            self.seed_password = None;
        }
    }

}

// View
impl<'a> EnterSeedPhrase {
    pub fn view(&self) -> Element<'a, AppMessage> {
        let header = common_elements::header_one("Enter seed phrase");

        let notification = widget::text(self.notification).size(12);

        let input_seed = self.input_seed();

        widget::column![header, notification, input_seed]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_x(iced::Alignment::Center)
            .spacing(50)
            .into()
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

        for index in 0..self.seed_phrase.nr_of_words() {
            if index % 4 == 0 && index != 0 {
                seed = seed.push(row);
                row = widget::row![]
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(20);
            }
            let word = self.seed_phrase.reference_word(index).unwrap_or("");

            let text_field = Self::seed_word_text_field_with_id(index, word);

            row = row.push(text_field);
        }
        seed.push(row)
    }

    fn seed_word_text_field_with_id(index: usize, word: &str) -> TextInput<'a, AppMessage> {
        seed_word_field(&format!("Word {}", index + 1), word)
            .id(Id::new(format!("{index}")))
            .on_input(move |input| Message::UpdateSingleWordInSeedPhrase(index, input).into())
            .on_paste(move |input| Message::UpdateMultipleWordsInSeedPhraseFromIndex(index, input).into())
    }
}
