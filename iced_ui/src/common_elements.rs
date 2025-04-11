use iced::widget::{self, Text};

pub fn header_one(text: &str) -> Text<'_> {
    widget::text(text).size(24).height(30)
}
