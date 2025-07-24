use deps::iced::{
    advanced::graphics::color,
    widget::pick_list::{Status, Style},
    Background, Border,
};

use crate::{styles::colors, Theme};

pub fn from_account(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let background_color = colors::layer_2(palette.background.base.color, palette.is_dark);

    let placeholder_color = colors::muted_light(palette.background.base.text);
    let text_color = palette.background.base.text;

    match status {
        Status::Active => Style {
            background: Background::Color(background_color),
            border: Border::default().rounded(5.),
            text_color: text_color,
            placeholder_color: placeholder_color,
            handle_color: text_color,
        },
        Status::Hovered => Style {
            background: Background::Color(colors::hover(background_color, palette.is_dark)),
            border: Border::default().rounded(5.),
            text_color: text_color,
            placeholder_color: placeholder_color,
            handle_color: text_color,
        },
        Status::Opened { is_hovered } => from_account(theme, Status::Active),
    }
}
