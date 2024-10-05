use iced::widget::{self, Text};

pub fn header_one(text: &str) -> Text<'_> {
    widget::text(text).size(20).height(30)
}
