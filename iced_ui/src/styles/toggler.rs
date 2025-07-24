use deps::iced::widget::toggler::{self, Status, Style};

use crate::Theme;
use std::default::Default;

use crate::styles::colors;

pub fn base_layer_1(theme: &Theme, status: Status) -> Style {
    let mut style = toggler::default(theme, status);
    let background = colors::lighten(theme.extended_palette().primary.base.color, 0.1);
    if let Status::Active { is_toggled } | Status::Hovered { is_toggled } = status {
        if is_toggled {
            style.background = background;
        }
    }
    style
}
