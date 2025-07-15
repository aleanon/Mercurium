use deps::iced::{
    advanced::graphics::core::widget,
    border::Radius,
    widget::text_editor::{self, Catalog, Status, Style},
    Background, Border,
};

use crate::{styles::colors, Theme};

pub fn primary(theme: &Theme, status: Status) -> Style {
    match status {
        Status::Active => Style {
            background: Background::Color(colors::background_card(theme)),
            border: Border {
                color: colors::border_subtle(theme),
                width: 1.,
                radius: Radius::new(5),
            },
            icon: colors::accent_primary_weak(theme),
            placeholder: colors::text_muted(theme),
            selection: colors::state_selection(theme),
            value: colors::text_primary(theme),
        },
        Status::Hovered => Style {
            border: Border {
                color: colors::border_default(theme),
                width: 1.,
                radius: Radius::new(5),
            },
            ..primary(theme, Status::Active)
        },
        Status::Focused { is_hovered } => Style {
            border: Border {
                color: colors::state_focus(theme),
                width: 1.,
                radius: Radius::new(5),
            },
            ..primary(theme, Status::Active)
        },
        Status::Disabled => text_editor::default(theme, Status::Disabled),
    }
    // let mut style = text_editor::default(theme, status);
    // style.background = Background::Color(colors::background_card(theme));
    // style.border = style.border.rounded(5);
    // style
}
