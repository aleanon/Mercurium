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

impl<'a> RestoreFromSeed {
    pub fn enter_seed_phrase_view(&self) -> Column<'a, AppMessage> {
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

        for index in 0..24 {
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
