use deps::*;
use no_mangle_if_debug::no_mangle_if_debug;

pub use iced::{widget::text::Style, Theme};

#[no_mangle_if_debug]
pub fn primary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.extended_palette().primary.weak.color),
    }
}
