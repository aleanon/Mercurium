use iced::{widget::text, Theme};

pub struct Text;

impl text::StyleSheet for Text {
    type Style = Theme;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        let palette = style.extended_palette();
        text::Appearance {
            color: Some(palette.primary.weak.color),
        }
    }
}
