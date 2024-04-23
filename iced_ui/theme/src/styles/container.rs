use iced::{
    border::Radius,
    color,
    gradient::{ColorStop, Linear},
    widget::{
        container,
        shader::wgpu::{naga::back, util::backend_bits_from_env},
    },
    Background, Border, Color, Gradient, Radians, Shadow, Theme, Vector,
};

pub struct MainWindow;

impl MainWindow {
    pub fn style(theme: &Theme) -> container::Appearance {
        let background = theme.extended_palette().background.base.color.inverse();

        container::Appearance {
            background: Some(iced::Background::Color(background)),
            ..Default::default()
        }
    }
}

pub struct MenuContainer;

impl MenuContainer {
    const BACKGROUND_ALPHA_STEP: f32 = 0.001;
    const MENU_ALPHA: f32 = 0.1;
    const STOPS_LEN: usize = 8;

    pub fn style(theme: &Theme) -> container::Appearance {
        let background_color = theme.extended_palette().background.base.color;
        let mut stops: [Option<ColorStop>; Self::STOPS_LEN] = [None; Self::STOPS_LEN];
        let mut current_alpha = background_color.a;
        let mut current_offset = 0.;

        for i in 0..Self::STOPS_LEN {
            let mut color = background_color.clone();
            color.a = current_alpha;
            stops[i] = Some(ColorStop {
                color,
                offset: current_offset,
            });
            current_offset += 0.12;
            current_alpha -= Self::BACKGROUND_ALPHA_STEP
        }

        let background = Some(Background::Gradient(Gradient::Linear(Linear {
            angle: Radians(1.570796),
            stops,
        })));

        container::Appearance {
            text_color: None,
            background,
            border: Border::default(),
            shadow: Shadow::default(),
        }
    }
}

pub struct CenterPanel;

impl CenterPanel {
    pub fn style(theme: &Theme) -> container::Appearance {
        let mut background = theme.extended_palette().background.base.color;
        background.a -= 0.004;

        container::Appearance {
            background: Some(iced::Background::Color(background)),
            ..Default::default()
        }
    }
}

pub struct AccountOverview;

impl AccountOverview {
    pub fn style(theme: &iced::Theme) -> container::Appearance {
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

        container::Appearance {
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 1.,
                radius: Radius::from([10.; 4]),
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
}

pub struct AssetListItem;

impl AssetListItem {
    pub fn style(theme: &Theme) -> container::Appearance {
        let background = theme.extended_palette().background.base;
        let mut background_color = background.color;
        background_color.a -= 0.01;

        container::Appearance {
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
}

// pub struct AddAssetContainer;

// impl container::StyleSheet for AddAssetContainer {
//     type Style = Theme;

//     fn appearance(&self, style: &Self::Style) -> container::Appearance {
//         container::Appearance { border: Border {} }
//     }
// }

pub struct OverlayContainer;

impl OverlayContainer {
    pub fn style(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();
        let mut background_color = palette.background.base.color;
        background_color.a = 0.2;

        container::Appearance {
            background: Some(Background::Color(background_color)),
            ..Default::default()
        }
    }
}

pub struct OverlayInner;

impl OverlayInner {
    pub fn style(theme: &Theme) -> container::Appearance {
        let mut appearance = CenterPanel::style(theme);
        appearance.border.radius = Radius::from(10.);
        appearance.border.width = 1.;
        appearance.border.color = theme.extended_palette().primary.weak.color;
        appearance
    }
}

pub struct NotificationSuccess;

impl NotificationSuccess {
    pub fn style(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();
        container::Appearance {
            background: Some(Background::Color(palette.success.weak.color)),
            text_color: Some(palette.success.weak.text),
            ..Default::default()
        }
    }
}

pub struct NotificationError;

impl NotificationError {
    pub fn style(theme: &Theme) -> container::Appearance {
        let palette = theme.extended_palette();
        container::Appearance {
            background: Some(Background::Color(palette.danger.weak.color)),
            text_color: Some(palette.danger.weak.text),
            ..Default::default()
        }
    }
}
