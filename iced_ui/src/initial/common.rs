use deps::*;

use iced::{
    widget::{self, text::LineHeight, Button, Row},
    Length,
};

use crate::{app::AppMessage, styles};

pub fn nav_button<'a, Message: Clone>(text: &'a str, on_press: Message) -> Button<'a, Message> {
    Button::new(
        widget::text(text)
            .size(20)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
    .width(80)
    .on_press(on_press)
    .style(styles::button::base_layer_2_rounded_with_shadow)
}

pub fn nav_row<'a, Message: Clone + 'a>(
    back: Button<'a, Message>,
    next: Button<'a, Message>,
) -> Row<'a, Message> {
    let space = widget::Space::with_width(Length::Fill);
    widget::row![back, space, next].align_y(iced::Alignment::Start)
}

pub fn seed_word_field<'a, Message: Clone>(
    placeholder: &str,
    input: &str,
) -> widget::TextInput<'a, Message> {
    widget::text_input(placeholder, input)
        .size(16)
        .width(100)
        .line_height(LineHeight::Relative(2.))
}

pub fn text_input_field<'a, Message: Clone + 'a>(
    placeholder: &str,
    input: &str,
) -> widget::TextInput<'a, Message> {
    widget::text_input(placeholder, input)
        .size(16)
        .width(250)
        .line_height(LineHeight::Relative(1.5))
}

pub fn notification_field(text: &str) -> widget::Text {
    widget::text(text).size(16).width(250)
}
