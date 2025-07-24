use deps::iced::{
    advanced::graphics::core::widget,
    border::Radius,
    widget::text_editor::{self, Catalog, Status, Style},
    Background, Border,
};

use crate::{styles::colors, Theme};

pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        Status::Active => Style {
            background: Background::Color(palette.background.weak.color),
            border: Border {
                color: palette.secondary.base.color,
                width: 1.,
                radius: Radius::new(5),
            },
            icon: palette.primary.base.color,
            placeholder: colors::muted_light(palette.background.weak.text),
            selection: colors::muted_light(palette.primary.weak.color),
            value: palette.background.base.text,
        },
        Status::Hovered => Style {
            border: Border {
                color: palette.background.strong.color,
                width: 1.,
                radius: Radius::new(5),
            },
            ..primary(theme, Status::Active)
        },
        Status::Focused { is_hovered } => Style {
            border: Border {
                color: colors::muted_light(palette.primary.weak.color),
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
