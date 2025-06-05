use deps::iced::{
    widget::pick_list::{Status, Style},
    Background, Border, Element, Theme,
};

pub fn from_account(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let mut background_color = palette.background.base.color;
    background_color.r -= 0.005;
    background_color.g -= 0.005;
    background_color.b -= 0.005;

    Style {
        background: Background::Color(background_color),
        border: Border::default().rounded(5.),
        text_color: palette.background.base.text,
        placeholder_color: palette.background.weak.text,
        handle_color: palette.background.base.text,
    }
}
