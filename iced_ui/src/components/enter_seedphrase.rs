use iced::{advanced::renderer::Style, widget::{self, text::LineHeight, text_input::Id, Column}, Length};
use types::crypto::SeedPhrase;

#[inline_tweak::tweak_fn]
pub fn input_seed<'a, Message>(
    seed_phrase: &SeedPhrase, 
    on_input: fn(usize, String) -> Message,
    on_paste: fn(usize, String) -> Message,
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
            .width(Length::Fill) 
            .line_height(LineHeight::Relative(2.))
            .id(Id::new(format!("{index}")))
            .style(styles::text_input::seed_word_input)
            .on_input(move |input| on_input(index, input))
            .on_paste(move |input| on_paste(index, input));
            
        let text_wrapper = widget::container(text_field)
            .style(styles::container::seed_word_wrapper);

        row = row.push(text_wrapper);
    }
    seed.push(row)
}
