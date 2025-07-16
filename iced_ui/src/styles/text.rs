use deps::{iced::advanced::graphics::color, *};

use iced::widget::text::Style;

use crate::styles::colors;
use crate::Theme;

pub fn primary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.extended_palette().primary.base.color),
    }
}

pub fn muted(theme: &Theme) -> Style {
    Style {
        color: Some(colors::muted(theme.extended_palette().background.base.text)),
    }
}

pub fn secondary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.extended_palette().secondary.base.color),
    }
}

pub fn warning(theme: &Theme) -> Style {
    Style {
        color: Some(theme.extended_palette().warning.base.color),
    }
}

pub fn error(theme: &Theme) -> Style {
    Style {
        color: Some(theme.extended_palette().danger.base.color),
    }
}
