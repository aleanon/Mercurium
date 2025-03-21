use iced::{
    border::Radius,
    widget::text_input::{default, Status, Style},
    Background, Border, Color, Theme,
};

pub fn general_input(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.border.radius = (5.).into();
    style
}

pub fn asset_amount(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    let palette = theme.extended_palette();
    style.border = Border {
        width: 0.,
        color: Color::TRANSPARENT,
        radius: Radius::new(10),
    };
    style.icon = Color::TRANSPARENT;
    style.background = Background::Color(palette.background.base.color);

    style
}

pub fn borderless(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.border = Border {
        width: 0.,
        color: Color::TRANSPARENT,
        ..Default::default()
    };
    style
}
