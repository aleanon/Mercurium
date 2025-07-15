use deps::iced::{border::Radius, overlay::menu::Style, Background, Border};

use crate::styles::colors;
use crate::Theme;

pub fn menu_primary(theme: &Theme) -> Style {
    Style {
        background: Background::Color(colors::background_primary(theme)),
        border: Border {
            color: colors::border_default(theme),
            width: 1.0,
            radius: Radius {
                bottom_left: 5.0,
                bottom_right: 5.0,
                top_left: 0.,
                top_right: 0.,
            },
        },
        selected_background: Background::Color(colors::state_hover(theme)),
        selected_text_color: colors::text_primary(theme),
        text_color: colors::text_secondary(theme),
    }
}
