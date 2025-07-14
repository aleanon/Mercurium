use deps::*;

use iced::{widget::text::Style, Theme};

use crate::styles::colors::dark;

pub fn primary(theme: &Theme) -> Style {
    Style {
        color: Some(dark::TEXT_PRIMARY),
    }
}

pub fn secondary(_theme: &Theme) -> Style {
    Style {
        color: Some(dark::TEXT_SECONDARY),
    }
}
