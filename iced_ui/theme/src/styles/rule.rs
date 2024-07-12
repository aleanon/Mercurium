use iced::{
    border::Radius,
    widget::rule::{FillMode, Style},
    Theme,
};

pub fn text_input_rule(theme: &Theme) -> Style {
    Style {
        radius: Radius::from([0., 0., 10., 10.]),
        fill_mode: FillMode::Full,
        width: 4,
        color: theme.extended_palette().primary.base.color,
    }
}
