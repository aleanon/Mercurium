use deps::*;

use iced::{
    border::Radius,
    widget::rule::{FillMode, Style},
};

use crate::styles::colors::dark;
use crate::Theme;

pub fn text_input_rule(theme: &Theme) -> Style {
    Style {
        radius: Radius {
            top_left: 0.,
            top_right: 0.,
            bottom_left: 10.,
            bottom_right: 10.,
        },
        fill_mode: FillMode::Full,
        color: dark::ACCENT_PRIMARY_BASE,
        snap: true,
    }
}
