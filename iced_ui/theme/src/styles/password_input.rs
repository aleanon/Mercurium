use iced::{border::Radius, widget::container::Style, Border, Color, Shadow, Theme};

pub fn container(theme: &Theme) -> Style {
    let background = theme.extended_palette().background.base;
    let mut background_color = background.color;
    background_color.a -= 0.01;

    Style {
        background: Some(iced::Background::Color(background_color)),
        border: Border {
            radius: Radius::from(0),
            color: Color::TRANSPARENT,
            width: 0.,
        },
        shadow: Shadow::default(),
        text_color: None,
    }
}