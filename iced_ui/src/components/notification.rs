use deps::*;

use iced::{widget::{self, container::Style, Container}, Background, Length, Theme};
use types::Notification;


pub fn notification<'a, Message: Clone>(notification: &'a Notification) -> Container<'a, Message> {
    Container::new(
        widget::Text::new(notification.message())
            .size(18)
    ).style(|theme| notification_style(theme, notification))
    .padding(10)
    .center_x(Length::Fill)
    .center_y(Length::Shrink)
    .into()
}

fn notification_style(theme: &Theme, notification: &Notification) -> Style {
    let palette = theme.extended_palette();
    let text_color;
    let background_color;
    match notification {
        Notification::Info(_) => {
            text_color = palette.secondary.base.text;
            background_color = palette.secondary.base.color;
        },
        Notification::Success(_) => {
            text_color = palette.success.base.text;
            background_color = palette.success.base.color;
        },
        Notification::Warn(_) => {
            text_color = palette.danger.weak.text;
            background_color = palette.danger.weak.color;
        },
        Notification::Danger(_) => {
            text_color = palette.danger.strong.text;
            background_color = palette.danger.strong.color;
        },
        Notification::None => {
            text_color = palette.background.base.text;
            background_color = palette.background.base.color;
        },
    };

    
    Style {
        background: Some(Background::Color(background_color)),
        text_color: Some(text_color),
        ..Default::default()
    }
}