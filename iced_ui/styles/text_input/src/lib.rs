use deps::*;
use no_mangle_if_debug::no_mangle_if_debug;

pub use iced::{
    border::Radius,
    widget::text_input::{default, Status, Style},
    Background, Border, Color, Theme,
};

#[no_mangle_if_debug]
pub fn seed_word_input(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.background = Background::Color(Color::TRANSPARENT);
    style.border = style.border.color(Color::TRANSPARENT);
    style
}

#[no_mangle_if_debug]
pub fn general_input(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.border.radius = (5.).into();
    style
}

#[no_mangle_if_debug]
pub fn asset_amount(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    let palette = theme.extended_palette();
    let mut background_color = palette.background.base.color;
    background_color.r -= 0.03;
    background_color.g -= 0.03;
    background_color.b -= 0.03;
    style.border = Border {
        width: 0.,
        color: Color::TRANSPARENT,
        radius: Radius::new(10),
    };
    style.icon = Color::TRANSPARENT;
    style.background = Background::Color(background_color);

    style
}

#[no_mangle_if_debug]
pub fn borderless(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.border = Border {
        width: 0.,
        color: Color::TRANSPARENT,
        ..Default::default()
    };
    style
}

#[no_mangle_if_debug]
pub fn transparent_borderless(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.background = Background::Color(Color::TRANSPARENT);
    style.border = Border {
        width: 0.,
        color: Color::TRANSPARENT,
        ..Default::default()
    };
    style
}
