use iced::{
    border::Radius,
    gradient::{ColorStop, Linear},
    widget::container::Style,
    Background, Border, Color, Gradient, Radians, Shadow, Theme, Vector,
};

pub fn main_window(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(
            theme.extended_palette().background.base.color.inverse(),
        )),
        ..Default::default()
    }
}

pub fn menu_container(theme: &Theme) -> Style {
    const BACKGROUND_ALPHA_STEP: f32 = 0.001;
    const MENU_ALPHA: f32 = 0.1;
    const STOPS_LEN: usize = 8;

    let background_color = theme.extended_palette().background.base.color;
    let mut stops: [Option<ColorStop>; STOPS_LEN] = [None; STOPS_LEN];
    let mut current_alpha = background_color.a;
    let mut current_offset = 0.;

    for i in 0..STOPS_LEN {
        let mut color = background_color.clone();
        color.a = current_alpha;
        stops[i] = Some(ColorStop {
            color,
            offset: current_offset,
        });
        current_offset += 0.12;
        current_alpha -= BACKGROUND_ALPHA_STEP
    }

    let background = Some(Background::Gradient(Gradient::Linear(Linear {
        angle: Radians(1.570796),
        stops,
    })));

    Style {
        text_color: None,
        background,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn center_panel(theme: &Theme) -> Style {
    let mut background = theme.extended_palette().background.base.color;
    background.a -= 0.004;

    Style {
        background: Some(iced::Background::Color(background)),
        ..Default::default()
    }
}

pub fn account_overview(theme: &Theme) -> Style {
    let extended_palette = theme.extended_palette();
    let background_base = extended_palette.background.base;
    let mut background_color = background_base.color;
    let shadow_color = extended_palette.background.weak.color;
    let text_color = background_base.text;
    background_color.a += 0.01;

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
            offset: Vector::new(2., 2.),
            blur_radius: 3.,
        },
        background: Some(iced::Background::Color(background_color)),
        text_color: Some(text_color),
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
    let mut appearance = center_panel(theme);
    appearance.border.radius = Radius::from(10.);
    appearance.border.width = 1.;
    appearance.border.color = theme.extended_palette().primary.weak.color;
    appearance
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
            color: Color::TRANSPARENT,
            width: 0.,
        },
        shadow: Shadow::default(),
        text_color: None,
    }
}