use deps::iced::{Background, Border, border::Radius, overlay::menu::Style};
use deps::iced::{Color, Shadow};

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
        shadow: Shadow {
            ..Default::default()
        },
        selected_background: Background::Color(palette.primary.weak.color),
        selected_text_color: palette.primary.weak.text,
        text_color: palette.background.weakest.text,
    }
}
