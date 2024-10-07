use iced::{
    widget::{self, text::LineHeight, Button, Row},
    Length,
};

use crate::app::AppMessage;

pub fn nav_button<'a>(text: &'a str) -> Button<'a, AppMessage> {
    Button::new(
        widget::text(text)
            .size(16)
            .width(50)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
    )
}

pub fn nav_row<'a>(
    back: Button<'a, AppMessage>,
    next: Button<'a, AppMessage>,
) -> Row<'a, AppMessage> {
    let space = widget::Space::with_width(Length::Fill);
    widget::row![back, space, next]
        .width(Length::Fill)
        .align_y(iced::Alignment::Start)
}

pub fn seed_word_field<'a>(placeholder: &str, input: &str) -> widget::TextInput<'a, AppMessage> {
    widget::text_input(placeholder, input)
        .size(16)
        .width(100)
        .line_height(LineHeight::Relative(2.))
}

pub fn text_input_field<'a>(placeholder: &str, input: &str) -> widget::TextInput<'a, AppMessage> {
    widget::text_input(placeholder, input)
        .size(16)
        .width(250)
        .line_height(LineHeight::Relative(1.5))
}

pub fn notification_field(text: &str) -> widget::Text {
    widget::text(text).size(16).width(250)
}
