use deps::*;

use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{alignment::Vertical, border::Radius, widget::{self, button, column, container::{self, Style}, row, text, text_input}, Border, Color, Length, Shadow, Theme};


pub fn password_input<'a, Message>(
    placeholder: &'static str,
    password: &str,
    view_password: bool,
    on_toggle: Message,
    on_input: fn(String) -> Message,
    on_submit: Message,
) -> container::Container<'a, Message>
where
    Message: Clone + 'a,
{
    let input = text_input(placeholder, password)
        .on_input(on_input)
        .on_paste(on_input)
        .on_submit(on_submit)
        .width(Length::FillPortion(9))
        .size(15)
        .secure(!view_password)
        .style(styles::text_input::borderless);

    let view_password_icon = if view_password {
        Bootstrap::EyeSlash
    } else {
        Bootstrap::Eye
    };

    let toggle_view_password = button(text(view_password_icon).font(BOOTSTRAP_FONT))
        .on_press(on_toggle)
        .width(Length::FillPortion(1))
        .padding(0)
        .style(button::text);

    let input_and_button = row![input, toggle_view_password]
        .align_y(Vertical::Center)
        .padding(2.)
        .spacing(5);

    let rule = widget::Rule::horizontal(1)
        .style(styles::rule::text_input_rule);

    let content = column![input_and_button, rule];

    widget::container(content)
        .style(styles::container::password_input)
}
