use deps::iced::widget::toggler::{Status, Style};

use crate::Theme;
use std::default::Default;

use crate::styles::colors::dark::{self, ACCENT_PRIMARY_BASE};

pub fn primary(theme: &Theme, status: Status) -> Style {
    match status {
        Status::Active { is_toggled } => {
            if is_toggled {
                Style {
                    background: dark::ACCENT_PRIMARY_BASE,
                    background_border_color: dark::BORDER_DEFAULT,
                    background_border_width: 1.0,
                    foreground: dark::TEXT_PRIMARY,
                    foreground_border_color: dark::BORDER_DEFAULT,
                    foreground_border_width: 1.0,
                }
            } else {
                Style {
                    background: dark::BACKGROUND_ELEVATED,
                    background_border_color: dark::BORDER_DEFAULT,
                    background_border_width: 1.0,
                    foreground: dark::BACKGROUND_PRIMARY,
                    foreground_border_color: dark::BORDER_DEFAULT,
                    foreground_border_width: 1.0,
                }
            }
        }
        Status::Disabled => Style {
            background: dark::BACKGROUND_DISABLED,
            background_border_color: dark::BORDER_SUBTLE,
            background_border_width: 1.0,
            foreground: dark::TEXT_MUTED,
            foreground_border_color: dark::BORDER_SUBTLE,
            foreground_border_width: 1.0,
        },
        Status::Hovered { is_toggled } => {
            if is_toggled {
                Style {
                    background: dark::ACCENT_PRIMARY_BASE,
                    background_border_color: dark::BORDER_DEFAULT,
                    background_border_width: 1.0,
                    foreground: dark::TEXT_MUTED,
                    foreground_border_color: dark::BORDER_DEFAULT,
                    foreground_border_width: 1.0,
                }
            } else {
                Style {
                    background: dark::BACKGROUND_ELEVATED,
                    background_border_color: dark::BORDER_DEFAULT,
                    background_border_width: 1.0,
                    foreground: dark::STATE_HOVER,
                    foreground_border_color: dark::BORDER_DEFAULT,
                    foreground_border_width: 1.0,
                }
            }
        }
    }
}
