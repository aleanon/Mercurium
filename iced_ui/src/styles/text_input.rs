use deps::{iced::theme::Palette, *};

use iced::{
    border::Radius,
    widget::text_input::{default, Status, Style},
    Background, Border, Color,
};

use crate::{styles::colors, Theme};

pub fn seed_word_input(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.background = Background::Color(Color::TRANSPARENT);
    style.border = style.border.color(Color::TRANSPARENT);
    style
}

/// For a text field on top of a secondary background.
pub fn secondary(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.background = Background::Color(theme.extended_palette().secondary.base.color);
    style
}

pub fn elevated(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let mut style = default(theme, status);
    style.background = Background::Color(palette.background.base.color);
    style.border = style
        .border
        .color(colors::muted_light(palette.background.base.text));
    style
}

pub fn general_input(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    style.border.radius = (5.).into();
    style
}

pub fn base_layer_1_rounded(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    let palette = theme.extended_palette();
    let background_color = colors::layer_1(palette.background.base.color, palette.is_dark);

    style.border = Border {
        width: 0.,
        color: Color::TRANSPARENT,
        radius: Radius::new(5),
    };
    style.icon = Color::TRANSPARENT;
    style.background = Background::Color(background_color);

    style
}

pub fn base_layer_2_rounded(theme: &Theme, status: Status) -> Style {
    let mut style = default(theme, status);
    let palette = theme.extended_palette();
    let background_color = colors::layer_2(palette.background.base.color, palette.is_dark);

    style.border = Border {
        width: 0.,
        color: Color::TRANSPARENT,
        radius: Radius::new(5),
    };
    style.icon = Color::TRANSPARENT;
    style.background = Background::Color(background_color);

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
