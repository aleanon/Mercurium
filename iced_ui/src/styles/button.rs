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

pub fn general_selected_button(theme: &Theme, status: Status) -> Style {
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

pub fn layer_2(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let background_color = colors::layer_2(palette.background.base.color, palette.is_dark);
    let text_color = palette.background.base.text;

    match status {
        Status::Active | Status::Pressed | Status::Disabled => Style {
            background: Some(Background::Color(background_color)),
            text_color,
            border: Border {
                radius: Radius::from(5),
                width: 0.,
                ..Default::default()
            },
            shadow: Shadow {
                offset: Vector::new(0., 0.),
                blur_radius: 3.,
                color: colors::shadow(background_color, palette.is_dark),
            },
            ..Default::default()
        },
        Status::Hovered => Style {
            background: Some(Background::Color(colors::hover(
                background_color,
                palette.is_dark,
            ))),
            text_color,
            ..layer_2(theme, Status::Active)
        },
    }
}

pub fn choose_recipient(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let background_color = colors::layer_2(palette.background.base.color, palette.is_dark);

    let style = Style {
        background: None,
        border: Border {
            radius: Radius::from(5.),
            ..Default::default()
        },
        text_color: palette.background.base.text,
        ..Default::default()
    };

    match status {
        Status::Active | Status::Pressed => style.with_background(background_color),
        Status::Hovered => style.with_background(palette.background.weak.color),
        Status::Disabled => style.with_background(palette.background.strong.color),
    }
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
            text_color: palette.primary.weak.color,
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

pub fn asset_list_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        Status::Active | Status::Pressed | Status::Disabled => {
            let mut background_color =
                colors::layer_2(palette.background.base.color, palette.is_dark);
            // background_color.r -= 0.02;
            // background_color.g -= 0.02;
            // background_color.b -= 0.02;

            Style {
                background: Some(Background::Color(background_color)),
                text_color: palette.background.base.text,
                ..Default::default()
            }
        }
        Status::Hovered => {
            let mut background_color = palette.background.weak.color;
            background_color.a = 0.1;

            Style {
                background: Some(Background::Color(background_color)),
                ..asset_list_button(theme, Status::Active)
            }
        }
    }
}

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
