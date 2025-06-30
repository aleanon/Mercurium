use deps::*;

use iced::{widget::text::Style, Theme};

pub fn primary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.extended_palette().primary.weak.color),
    }
}
