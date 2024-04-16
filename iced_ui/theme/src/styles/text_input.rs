use iced::{
    border::Radius,
    theme,
    widget::text_input::{self, StyleSheet},
    Border, Color, Theme,
};

pub struct GeneralInput;

impl text_input::StyleSheet for GeneralInput {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        let mut appearance = style.active(&theme::TextInput::Default);
        appearance.border.radius = Radius::from(5.);
        appearance
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        let mut appearance = style.disabled(&theme::TextInput::Default);
        appearance.border.radius = Radius::from(5.);
        appearance
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        let mut appearance = style.focused(&theme::TextInput::Default);
        appearance.border.radius = Radius::from(5.);
        appearance
    }

    fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
        let mut appearance = style.hovered(&theme::TextInput::Default);
        appearance.border.radius = Radius::from(5.);
        appearance
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        style.disabled_color(&iced::theme::TextInput::Default)
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        style.placeholder_color(&iced::theme::TextInput::Default)
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        style.selection_color(&iced::theme::TextInput::Default)
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        style.value_color(&iced::theme::TextInput::Default)
    }
}

pub struct AssetAmount;

impl text_input::StyleSheet for AssetAmount {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        let extended_palette = style.extended_palette();
        let background = extended_palette.background.base.color;
        let border = Border {
            width: 0.,
            color: Color::TRANSPARENT,
            radius: Radius::from([10., 10., 10., 10.]),
        };

        text_input::Appearance {
            background: iced::Background::Color(background),
            border,
            icon_color: Color::TRANSPARENT,
        }
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        style.extended_palette().background.weak.color
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }

    fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        style.extended_palette().background.base.text
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        style.extended_palette().background.weak.text
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        style.extended_palette().background.strong.color
    }
}
