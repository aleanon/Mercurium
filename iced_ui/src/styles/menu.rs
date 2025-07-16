use deps::iced::Color;
use deps::iced::{border::Radius, overlay::menu::Style, Background, Border};

use crate::styles::colors;
use crate::Theme;

pub fn primary(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Background::Color(palette.background.weakest.color),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius {
                bottom_left: 5.0,
                bottom_right: 5.0,
                top_left: 0.,
                top_right: 0.,
            },
        },
        selected_background: Background::Color(palette.primary.weak.color),
        selected_text_color: palette.primary.weak.text,
        text_color: palette.background.weakest.text,
    }
}
