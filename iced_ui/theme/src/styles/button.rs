use iced::{border::Radius, widget::button, Background, Border, Color, Shadow, Theme, Vector};

pub struct GeneralSelectedButton;

impl button::StyleSheet for GeneralSelectedButton {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = style.extended_palette();

        button::Appearance {
            background: Some(Background::Color(palette.secondary.base.color)),
            text_color: palette.secondary.base.text,
            border: Border {
                radius: Radius::from(10.),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub struct GeneralButton;

impl button::StyleSheet for GeneralButton {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = style.extended_palette();

        button::Appearance {
            background: Some(Background::Color(palette.primary.weak.color)),
            text_color: palette.primary.weak.text,
            border: Border {
                radius: Radius::from(10),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let palette = style.extended_palette();

        button::Appearance {
            background: Some(Background::Color(palette.primary.strong.color)),
            text_color: palette.primary.strong.text,
            ..self.active(style)
        }
    }
}

pub struct ChooseAccount;

impl button::StyleSheet for ChooseAccount {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = style.extended_palette();
        let background;

        if palette.is_dark {
            background = palette.primary.weak.color;
        } else {
            background = palette.primary.strong.color;
        }

        button::Appearance {
            background: Some(Background::Color(background)),
            border: Border {
                radius: Radius::from([10., 10., 0., 0.]),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub struct MenuButton;

impl button::StyleSheet for MenuButton {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let text_color = style.extended_palette().background.base.text;

        button::Appearance {
            shadow_offset: iced::Vector { x: 0., y: 0. },
            background: None,
            text_color,
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
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let text_color = style.extended_palette().background.strong.text;

        button::Appearance {
            text_color,
            ..self.active(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let text_color = style.extended_palette().background.weak.text;

        button::Appearance {
            text_color,
            ..self.active(style)
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.hovered(style)
    }
}

pub struct SelectedMenuButton;

impl button::StyleSheet for SelectedMenuButton {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let mut background_color = style.extended_palette().background.weak.color;
        background_color.a = 0.1;

        button::Appearance {
            background: Some(Background::Color(background_color)),
            ..MenuButton.hovered(style)
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

pub struct AccountButton;

impl button::StyleSheet for AccountButton {
    type Style = Theme;
    fn active(&self, style: &Self::Style) -> button::Appearance {
        let extended_palette = style.extended_palette();
        let mut background_color = extended_palette.background.base.color;
        let shadow_color = extended_palette.background.weak.color;
        let text_color = extended_palette.background.base.text;
        background_color.a -= 0.1;

        button::Appearance {
            background: Some(iced::Background::Color(background_color)),
            text_color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 1.,
                radius: Radius::from([10.; 4]),
            },
            shadow: Shadow {
                color: shadow_color,
                offset: Vector::new(2., 2.),
                blur_radius: 3.,
            },
            shadow_offset: Vector { x: 0., y: 0. },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let extended_palette = style.extended_palette();
        let mut background_color = extended_palette.background.weak.color;
        let _text_color = extended_palette.background.base.text;
        background_color.a = 0.1;

        button::Appearance {
            background: Some(iced::Background::Color(background_color)),
            ..self.active(style)
        }
    }
}

pub struct AssetListButton;

impl button::StyleSheet for AssetListButton {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let background = style.extended_palette().background.base;
        let mut background_color = background.color;
        let text_color = background.text;
        background_color.a -= 0.01;

        button::Appearance {
            background: Some(Background::Color(background_color)),
            text_color: text_color,
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut background_color = style.extended_palette().background.weak.color;
        background_color.a = 0.1;

        button::Appearance {
            background: Some(Background::Color(background_color)),
            ..self.active(style)
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}
