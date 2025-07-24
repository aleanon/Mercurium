use deps::iced::{self, theme::Palette, widget::container::background};

use iced::{
    border::Radius,
    widget::button::{Status, Style},
    Background, Border, Color, Shadow, Vector,
};

use crate::styles::colors;
use crate::Theme;

pub fn setup_selection(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        Status::Active | Status::Pressed | Status::Disabled => Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            text_color: palette.primary.weak.text,
            border: Border {
                radius: Radius::from(3),
                ..Default::default()
            },
            shadow: Shadow {
                color: Color::BLACK,
                offset: Vector::ZERO,
                blur_radius: 10.,
            },
            ..Default::default()
        },
        Status::Hovered => Style {
            background: Some(Background::Color(palette.primary.weak.color)),
            text_color: palette.primary.strong.text,
            ..setup_selection(theme, Status::Active)
        },
    }
}

pub fn base_layer_2_selected(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        Status::Active | Status::Pressed | Status::Disabled | Status::Hovered => Style {
            background: Some(Background::Color(palette.secondary.base.color)),
            text_color: palette.secondary.base.text,
            border: Border {
                radius: Radius::from(5.),
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

pub fn primary(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        Status::Active | Status::Pressed => Style {
            background: Some(Background::Color(palette.primary.base.color)),
            text_color: palette.primary.base.text,
            border: Border {
                radius: Radius::from(5.),
                ..Default::default()
            },
            ..Default::default()
        },
        Status::Disabled => {
            primary(theme, Status::Active).with_background(palette.primary.weak.color)
        }
        Status::Hovered => {
            primary(theme, Status::Active).with_background(palette.primary.strong.color)
        }
    }
}

pub fn base_layer_1(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let background_color = colors::layer_1(palette.background.base.color, palette.is_dark);
    let text_color = palette.background.base.text;

    match status {
        Status::Active | Status::Pressed => Style {
            background: Some(Background::Color(background_color)),
            text_color,
            ..Default::default()
        },
        Status::Hovered => Style {
            background: Some(Background::Color(colors::hover(
                background_color,
                palette.is_dark,
            ))),
            text_color,
            ..Default::default()
        },
        Status::Disabled => Style {
            background: Some(Background::Color(colors::disabled(
                background_color,
                palette.is_dark,
            ))),
            text_color: colors::muted_max(text_color),
            ..Default::default()
        },
    }
}

pub fn base_layer_1_rounded_with_shadow(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let mut style = base_layer_1(theme, status);
    style.border = Border {
        radius: Radius::from(5),
        width: 0.,
        ..Default::default()
    };
    style.shadow = Shadow {
        offset: Vector::new(0., 0.),
        blur_radius: 3.,
        color: colors::shadow(
            colors::layer_1(palette.background.base.color, palette.is_dark),
            palette.is_dark,
        ),
    };
    style
}

pub fn base_layer_2(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let background_color = colors::layer_2(palette.background.base.color, palette.is_dark);
    let text_color = palette.background.base.text;

    match status {
        Status::Active | Status::Pressed => Style {
            background: Some(Background::Color(background_color)),
            text_color,
            ..Default::default()
        },
        Status::Hovered => Style {
            background: Some(Background::Color(colors::hover(
                background_color,
                palette.is_dark,
            ))),
            text_color,
            ..Default::default()
        },
        Status::Disabled => Style {
            background: Some(Background::Color(colors::disabled(
                background_color,
                palette.is_dark,
            ))),
            text_color: colors::muted_max(text_color),
            ..Default::default()
        },
    }
}

pub fn base_layer_2_rounded_with_shadow(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let mut style = base_layer_2(theme, status);
    style.border = Border {
        radius: Radius::from(5),
        width: 0.,
        ..Default::default()
    };
    style.shadow = Shadow {
        offset: Vector::new(0., 0.),
        blur_radius: 3.,
        color: colors::shadow(
            colors::layer_2(palette.background.base.color, palette.is_dark),
            palette.is_dark,
        ),
    };
    style
}

pub fn menu_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        Status::Active => Style {
            background: None,
            text_color: palette.background.base.text,
            border: Border {
                color: Color::TRANSPARENT,
                radius: Radius::from(10.),
                width: 0.,
            },
            shadow: Shadow {
                color: Color::TRANSPARENT,
                offset: Vector::ZERO,
                blur_radius: 0.,
            },
            ..Default::default()
        },
        Status::Hovered | Status::Pressed => Style {
            text_color: colors::lighten(palette.primary.base.color, 0.1),
            ..menu_button(theme, Status::Active)
        },
        Status::Disabled => Style {
            text_color: palette.background.weak.text,
            ..menu_button(theme, Status::Active)
        },
    }
}

pub fn selected_menu_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        _ => Style {
            background: Some(Background::Color(palette.background.weak.color)),
            ..menu_button(theme, Status::Hovered)
        },
    }
}

// pub fn asset_list_button(theme: &Theme, status: Status) -> Style {
//     let palette = theme.extended_palette();
//     match status {
//         Status::Active | Status::Pressed | Status::Disabled => {
//             let background_color = colors::layer_2(palette.background.base.color, palette.is_dark);

//             Style {
//                 background: Some(Background::Color(background_color)),
//                 text_color: palette.background.base.text,
//                 ..Default::default()
//             }
//         }
//         Status::Hovered => {
//             let mut background_color = palette.background.weak.color;
//             background_color.a = 0.1;

//             Style {
//                 background: Some(Background::Color(background_color)),
//                 ..asset_list_button(theme, Status::Active)
//             }
//         }
//     }
// }

pub fn nft_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        Status::Active | Status::Pressed | Status::Disabled => {
            let mut background_color = palette.background.weakest.color;
            background_color.r -= 0.02;
            background_color.g -= 0.02;
            background_color.b -= 0.02;

            Style {
                background: Some(Background::Color(background_color)),
                text_color: palette.background.base.text,
                border: Border {
                    radius: Radius::from(5.),
                    color: Color::TRANSPARENT,
                    width: 0.,
                },
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::new(0., 0.),
                    blur_radius: 3.,
                },
                ..Default::default()
            }
        }
        Status::Hovered => {
            let mut background_color = palette.background.weak.color;
            background_color.a = 0.1;

            Style {
                background: Some(Background::Color(background_color)),
                ..nft_button(theme, Status::Active)
            }
        }
    }
}
