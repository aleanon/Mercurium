use iced::{widget::{self, column}, Element, Length, Task};
use types::{crypto::{Password, SeedPhrase}, AppError, Notification};
use wallet::{wallet::Wallet, Setup};
use zeroize::Zeroize;

use crate::{common_elements, components, initial::common::{nav_button, nav_row}};

#[derive(Clone)]
pub enum Message {
    Back,
    Next,
    SetNotification(Notification),
    ClearNotification,
    InputSeedWords(usize, String),
    ToggleSeedPassword,
    InputSeedPassword(String),
}


#[derive(Debug)]
pub struct EnterSeedPhrase {
    pub notification: Notification,
    seed_phrase: SeedPhrase,
    seed_password: Option<Password>,
}

impl EnterSeedPhrase {
    pub fn new(wallet: &Wallet<Setup>, notification: Notification) -> Self {
        let seed_phrase = match wallet.seed_phrase() {
            Some(phrase) => SeedPhrase::from_str(phrase),
            None => SeedPhrase::new(),
        };

        let seed_password = match wallet.seed_password() {
            Some(password) => Some(Password::from(password)),
            None => None,
        };

        Self {
            notification,
            seed_phrase,
            seed_password,
        }
    }

    pub fn update(&mut self, message: Message, wallet: &mut Wallet<Setup>) -> Task<Message> {
        match message {
            Message::SetNotification(notification) => self.notification = notification,
            Message::ClearNotification => self.notification = Notification::None,
            Message::InputSeedWords(index, input) => self.update_multiple_words_in_seed_phrase_from_index(index, input),
            Message::ToggleSeedPassword => self.toggle_seed_password(wallet),
            Message::InputSeedPassword(input) => self.input_seed_password(input),
            Message::Back | Message::Next => {/*Handled in parent*/}
        }
        Task::none()
    }

    fn update_multiple_words_in_seed_phrase_from_index(&mut self, mut word_index: usize, mut input: String) {
        let mut words = input.split_ascii_whitespace();

        while let Some(word) = words.next() && word_index < self.seed_phrase.nr_of_words() {
            self.seed_phrase.update_word(word_index, &word);
            word_index += 1;
        }
        input.zeroize();
        self.notification = Notification::None;
    }

    fn toggle_seed_password(&mut self, wallet: &Wallet<Setup>) {
        self.seed_password = match self.seed_password {
            Some(_) => None,
            None => Some(Password::from(wallet.seed_password().unwrap_or("")))
        };
    }

    fn input_seed_password(&mut self, mut input: String) {
        if let Some(password) = &mut self.seed_password {
            password.replace(input.as_str());
        }
        input.zeroize();
    }

    pub fn save_to_wallet(&mut self, wallet: &mut Wallet<Setup>) -> Result<(), AppError> {
        wallet.set_seed_phrase_and_password(self.seed_phrase.phrase(), self.seed_password.clone())
            .map_err(|err| AppError::NonFatal(types::Notification::Info(err.to_string())))
    }
}

impl<'a> EnterSeedPhrase {
    pub fn view(&'a self) -> Element<'a, Message> {
        let header = common_elements::header_one("Enter seed phrase");

        let notification = components::notification::notification(&self.notification);

        let input_seed = components::enter_seedphrase::input_seed(&self.seed_phrase, 
            |index, input| Message::InputSeedWords(index, input));

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