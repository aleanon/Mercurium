use iced::{widget::{self, text::LineHeight, text_input::Id, Column}, Length};
use types::crypto::SeedPhrase;

pub fn input_seed<'a, Message>(
    seed_phrase: &SeedPhrase, 
    on_input: fn(usize, String) -> Message, 
    ) -> Column<'a, Message> 
    where
        Message: Clone + 'a,
    {
    let mut seed = widget::column![]
        .width(Length::Shrink)
        .height(Length::Shrink)
        .spacing(20);
    let mut row = widget::row![]
        .width(Length::Shrink)
        .height(Length::Shrink)
        .spacing(20);

    for index in 0..seed_phrase.nr_of_words() {
        if index % 4 == 0 && index != 0 {
            seed = seed.push(row);
            row = widget::row![]
                .width(Length::Shrink)
                .height(Length::Shrink)
                .spacing(20);
        }
        let word = seed_phrase.reference_word(index).unwrap_or("");

        let text_field = widget::text_input(&format!("Word {}", index + 1), word)
            .size(16)
            .width(100)
            .line_height(LineHeight::Relative(2.))
            .id(Id::new(format!("{index}")))
            .on_input(move |input| on_input(index, input))
            .on_paste(move |input| on_input(index, input));
            

        row = row.push(text_field);
    }
    seed.push(row)
}

// fn seed_word_text_field_with_id<'a, Message>(
//     index: usize, 
//     word: &str, 
//     on_input: fn(usize, String) -> Message, 
//     ) -> TextInput<'a, Message> 
//     where 
//     Message: Clone + 'a,
//     {
//     seed_word_field(&format!("Word {}", index + 1), word)
//         .id(Id::new(format!("{index}")))
//         .on_input(move |input| on_input(index, input))
//         .on_paste(move |input| on_input(index, input))
// }

// fn seed_word_field<'a, Message: Clone>(placeholder: &str, input: &str) -> widget::TextInput<'a, Message> {
//     widget::text_input(placeholder, input)
//         .size(16)
//         .width(100)
//         .line_height(LineHeight::Relative(2.))
// }