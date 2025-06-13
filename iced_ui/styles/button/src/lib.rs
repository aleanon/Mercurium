use deps::{
    iced::{theme::palette, widget::container::background},
    *,
};
use no_mangle_if_debug::no_mangle_if_debug;

use iced::{border::Radius, Background, Border, Color, Shadow, Vector};

pub use iced::{
    widget::button::{Status, Style},
    Theme,
};

#[no_mangle_if_debug]
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

#[no_mangle_if_debug]
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

#[no_mangle_if_debug]
pub fn general_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let mut background_color = palette.background.weakest.color;
    background_color.r -= 0.05;
    background_color.g -= 0.05;
    background_color.b -= 0.05;

    let shadow_color = palette.background.base.color;
    match status {
        Status::Active | Status::Pressed | Status::Disabled => Style {
            background: Some(Background::Color(background_color)),
            text_color: palette.primary.weak.text,
            border: Border {
                radius: Radius::from(5),
                width: 0.,
                ..Default::default()
            },
            shadow: Shadow {
                offset: Vector::new(0., 2.),
                blur_radius: 10.,
                color: shadow_color,
            },
            ..Default::default()
        },
        Status::Hovered => Style {
            background: Some(Background::Color(palette.primary.strong.color)),
            text_color: palette.primary.strong.text,
            ..general_button(theme, Status::Active)
        },
    }
}

#[no_mangle_if_debug]
pub fn choose_account(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let mut background_color = palette.background.base.color;
    background_color.r -= 0.03;
    background_color.g -= 0.03;
    background_color.b -= 0.03;

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
        Status::Active | Status::Pressed => {
            background_color.r -= 0.03;
            background_color.g -= 0.03;
            background_color.b -= 0.03;
            style.with_background(background_color)
        }
        Status::Hovered => style.with_background(background_color),
        Status::Disabled => style.with_background(palette.background.weak.color),
    }
}

#[no_mangle_if_debug]
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

#[no_mangle_if_debug]
pub fn selected_menu_button(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    match status {
        _ => Style {
            background: Some(Background::Color(palette.background.weak.color)),
            ..menu_button(theme, Status::Hovered)
        },
    }
}

#[no_mangle_if_debug]
pub fn account_button(theme: &Theme, status: Status) -> Style {
    let ext_palette = theme.extended_palette();
    let mut background_color = ext_palette.background.weakest.color;
    background_color.r -= 0.02;
    background_color.g -= 0.02;
    background_color.b -= 0.02;
    let shadow_color = ext_palette.background.base.color;

    match status {
        Status::Active | Status::Pressed | Status::Disabled => {
            // background_color.a += 0.1;
            Style {
                background: Some(iced::Background::Color(background_color)),
                text_color: ext_palette.background.base.text,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.,
                    radius: Radius::new(5),
                },
                shadow: Shadow {
                    color: shadow_color,
                    offset: Vector::new(0., 0.),
                    blur_radius: 5.,
                },
                ..Default::default()
            }
        }
        Status::Hovered => {
            background_color.r += 0.04;
            background_color.g += 0.04;
            background_color.b += 0.04;
            Style {
                background: Some(Background::Color(background_color)),
                ..account_button(theme, Status::Active)
            }
        }
    }
}

#[no_mangle_if_debug]
pub fn asset_list_button(theme: &Theme, status: Status) -> Style {
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
