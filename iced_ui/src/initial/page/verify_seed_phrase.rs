use deps::*;

use iced::{widget::{self, column}, Element, Length, Task};
use types::{crypto::SeedPhrase, Notification};
use wallet::{wallet::Wallet, Setup};
use zeroize::Zeroize;

use crate::{common_elements, components, initial::common::{nav_button, nav_row}};

#[derive(Clone)]
pub enum Message {
    Back,
    Next,
    InputSeedWord(usize, String),
    PasteSeedWords(usize, String),
}


#[derive(Debug)]
pub struct VerifySeedPhrase {
    notification: Notification,
    seed_phrase: SeedPhrase,
    verify_seed_phrase: SeedPhrase,
}

impl VerifySeedPhrase {
    pub fn new(wallet: &Wallet<wallet::Setup>, notification: Notification) -> Self {
        let Some(phrase) = wallet.seed_phrase() else {
            unreachable!("No Seed phrase in wallet")
        };

        let seed_phrase = SeedPhrase::from_str(phrase);

        Self {
            notification,
            seed_phrase,
            verify_seed_phrase: SeedPhrase::new(),
        }
    }

    pub fn update(&mut self, message: Message, wallet: &mut Wallet<Setup>) -> Task<Message> {
        match message {
            Message::InputSeedWord(index, input) => self.input_seed_word(index, input),
            Message::PasteSeedWords(index, input) => self.update_multiple_words_in_seed_phrase_from_index(index, input),
            Message::Back | Message::Next => {/*Handled in parent*/}
        }
        Task::none()
    }

    fn input_seed_word(&mut self, index: usize, mut input: String) {
        self.seed_phrase.update_word(index, &input);
        input.zeroize();
    }

    fn update_multiple_words_in_seed_phrase_from_index(&mut self, mut word_index: usize, mut input: String) {
        let mut words = input.split_ascii_whitespace();

        while let Some(word) = words.next() && word_index < self.seed_phrase.nr_of_words() {
            self.seed_phrase.update_word(word_index, &word);
            word_index += 1;
        }
        input.zeroize();
    }

    pub fn seed_phrase_is_correct(&self) -> bool {
        self.seed_phrase == self.verify_seed_phrase
    }

    pub fn notify_input_state(&mut self) {
        self.notification = Notification::Info("Seed phrase does not match".to_string())
    }

}

impl<'a> VerifySeedPhrase {
    pub fn view(&'a self) -> Element<'a, Message> {
        let header = common_elements::header_one("Retype Seed Phrase");

        let notification = components::notification::notification(&self.notification);

        let input_seed = components::enter_seedphrase::input_seed(
            &self.verify_seed_phrase, 
            Message::PasteSeedWords,
            Message::PasteSeedWords);

        let content = widget::column![header, notification, input_seed]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_x(iced::Alignment::Center)
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