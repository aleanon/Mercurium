use deps::iced;
use iced::{
    border::Radius, widget::container::Style, Background, Border, Color, Shadow, Theme, Vector,
};

pub fn main_window(theme: &Theme) -> Style {
    // Style::default()
    // style.background = Some(Background::Color(Color::from_rgb8(50, 50, 50)));
    let mut style = Style::default();
    let palette = theme.extended_palette();
    let background_color = palette.background.weakest.color;
    style.background = Some(Background::Color(background_color));
    style
    // style
}

pub fn menu_container(theme: &Theme) -> Style {
    let mut style = Style::default();
    let palette = theme.extended_palette();
    let mut background_color = palette.background.weakest.color;
    background_color.r -= 0.005;
    background_color.g -= 0.005;
    background_color.b -= 0.005;

    style.background = Some(Background::Color(background_color));
    // style

    // style.background = Some(Background::Color(Color::from_rgb8(40, 40, 40)));
    style.shadow = Shadow {
        color: palette.background.base.color,
        offset: Vector::new(0.0, 0.0),
        blur_radius: 5.,
    };
    style
    // const BACKGROUND_ALPHA_STEP: f32 = 0.001;
    // const MENU_ALPHA: f32 = 0.1;
    // const STOPS_LEN: usize = 8;

    // let background_color = theme.extended_palette().background.base.color;
    // let mut stops: [Option<ColorStop>; STOPS_LEN] = [None; STOPS_LEN];
    // let mut current_alpha = background_color.a;
    // let mut current_offset = 0.;

    // for i in 0..STOPS_LEN {
    //     let mut color = background_color.clone();
    //     color.a = current_alpha;
    //     stops[i] = Some(ColorStop {
    //         color,
    //         offset: current_offset,
    //     });
    //     current_offset += 0.12;
    //     current_alpha -= BACKGROUND_ALPHA_STEP
    // }

    // let background = Some(Background::Gradient(Gradient::Linear(Linear {
    //     angle: Radians(1.570796),
    //     stops,
    // })));

    // Style {
    //     text_color: None,
    //     background,
    //     border: Border::default(),
    //     shadow: Shadow::default(),
    // }
}

pub fn center_panel(theme: &Theme) -> Style {
    // let mut background = theme.extended_palette().background.base.color;
    // background.a -= 0.004;

    // Style {
    //     background: Some(iced::Background::Color(background)),
    //     ..Default::default()
    // }

    Style::default()
}

pub fn token_container(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut background_color = palette.background.weakest.color;
    background_color.r -= 0.005;
    background_color.g -= 0.005;
    background_color.b -= 0.005;

    let shadow_color = palette.background.base.color;

    Style {
        background: Some(Background::Color(background_color)),
        border: Border::default().rounded(5),
        shadow: Shadow {
            color: shadow_color,
            blur_radius: 5.,
            offset: Vector::new(0., 0.),
        },
        ..Default::default()
    }
}

pub fn seed_word_wrapper(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let background_color = Color::from_rgb8(40, 40, 40);
    let mut shadow = Shadow::default();
    shadow.color = Color::BLACK;
    shadow.blur_radius = 2.;
    Style {
        background: Some(Background::Color(background_color)),
        border: Border::default().rounded(5),
        text_color: Some(palette.secondary.base.text.inverse()),
        shadow: shadow,
        ..Default::default()
    }
}

pub fn account_overview(theme: &Theme) -> Style {
    let extended_palette = theme.extended_palette();
    let background_base = extended_palette.background.base;
    let mut background_color = Color::from_rgb8(40, 40, 40);
    let shadow_color = Color::BLACK;
    let text_color = background_base.text;
    // background_color.r.checked_add_assign(&20.).ok();
    // background_color.g.checked_add_assign(&20.).ok();
    // background_color.b.checked_add_assign(&20.).ok();

    // for c in &mut background_color[0..3] {
    //   if let Some(num) = c.checked_add(2) {
    //     *c = num
    //   } else {
    //     *c = 0
    //   }
    // }

    Style {
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 1.,
            radius: Radius::new(10),
        },
        shadow: iced::Shadow {
            color: shadow_color,
            offset: Vector::new(0., 0.),
            blur_radius: 3.,
        },
        background: Some(iced::Background::Color(background_color)),
        text_color: Some(text_color),
        ..Default::default()
    }
}

pub fn asset_list_item(theme: &Theme) -> Style {
    let background = theme.extended_palette().background.base;
    let mut background_color = background.color;
    background_color.a -= 0.01;

    Style {
        background: Some(iced::Background::Color(background_color)),
        border: Border {
            radius: Radius::from(0),
            color: Color::TRANSPARENT,
            width: 0.,
        },
        shadow: Shadow::default(),
        text_color: None,
        ..Default::default()
    }
}

pub fn overlay_container(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut background_color = palette.background.base.color;
    background_color.a = 0.2;

    Style {
        background: Some(Background::Color(background_color)),
        ..Default::default()
    }
}

pub fn overlay_inner(theme: &Theme) -> Style {
    let mut style = center_panel(theme);
    style.border.radius = Radius::from(10.);
    style.border.width = 1.;
    style.border.color = theme.extended_palette().primary.weak.color;
    style
}

pub fn notification_success(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.success.weak.color)),
        text_color: Some(palette.success.weak.text),
        ..Default::default()
    }
}

pub fn notification_error(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        background: Some(Background::Color(palette.danger.weak.color)),
        text_color: Some(palette.danger.weak.text),
        ..Default::default()
    }
}

pub fn password_input(theme: &Theme) -> Style {
    Style {
        background: None,
        border: Border {
            radius: Radius::from(10),
            color: Color::WHITE,
            width: 0.,
        },
        shadow: Shadow::default(),
        text_color: None,
        ..Default::default()
    }
}

pub fn nft_card(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut background_color = palette.background.weak.color;
    background_color.r -= 0.005;
    background_color.g -= 0.005;
    background_color.b -= 0.005;

    let shadow_color = Color::BLACK;

    Style {
        background: Some(Background::Color(background_color)),
        text_color: Some(palette.background.base.text),
        border: Border {
            radius: Radius::from(5.),
            color: Color::TRANSPARENT,
            width: 0.,
        },
        shadow: Shadow {
            color: shadow_color,
            offset: Vector::new(0., 0.),
            blur_radius: 3.,
        },
        ..Default::default()
    }
}

pub fn tag(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    let mut background_color = palette.background.base.color;
    background_color.r -= 0.005;
    background_color.g -= 0.005;
    background_color.b -= 0.005;

    Style {
        background: Some(Background::Color(background_color)),
        text_color: Some(palette.background.base.text),
        border: Border {
            radius: Radius {
                bottom_right: 50.,
                top_right: 50.,
                ..Default::default()
            },
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

pub fn recipient(theme: &Theme) -> Style {
    let extended_palette = theme.extended_palette();
    let background_base = extended_palette.background.base;
    let mut background_color = background_base.color;
    background_color.r -= 0.005;
    background_color.g -= 0.005;
    background_color.b -= 0.005;

    let shadow_color = Color::BLACK;
    let text_color = background_base.text;

    Style {
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 1.,
            radius: Radius::new(5),
        },
        shadow: iced::Shadow {
            color: shadow_color,
            offset: Vector::new(0., 0.),
            blur_radius: 3.,
        },
        background: Some(iced::Background::Color(background_color)),
        text_color: Some(text_color),
        ..Default::default()
    }
}
