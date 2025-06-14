use deps::*;
use no_mangle_if_debug::no_mangle_if_debug;

pub use iced::{
    border::Radius,
    widget::rule::{FillMode, Style},
    Theme,
};

#[no_mangle_if_debug]
pub fn text_input_rule(theme: &Theme) -> Style {
    Style {
        radius: Radius {
            top_left: 0.,
            top_right: 0.,
            bottom_left: 10.,
            bottom_right: 10.,
        },
        fill_mode: FillMode::Full,
        width: 3,
        color: theme.extended_palette().primary.base.color,
        snap: true,
    }
}
