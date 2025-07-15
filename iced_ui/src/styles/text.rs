use deps::{iced::advanced::graphics::color, *};

use iced::widget::text::Style;

use crate::styles::colors::{self, dark};
use crate::Theme;

pub fn primary(theme: &Theme) -> Style {
    Style {
        color: Some(colors::text_primary(theme)),
    }
}

pub fn secondary(theme: &Theme) -> Style {
    Style {
        color: Some(colors::text_secondary(theme)),
    }
}

pub fn error(theme: &Theme) -> Style {
    Style {
        color: Some(colors::accent_error(theme)),
    }
}
