use deps::iced::{theme::palette::Extended, Color};

pub fn shadow(background_color: Color, is_dark: bool) -> Color {
    if is_dark {
        background_color
    } else {
        Color::BLACK
    }
}

pub fn darken(mut color: Color, amount: f32) -> Color {
    color.r = (color.r - amount).max(0.0);
    color.g = (color.g - amount).max(0.0);
    color.b = (color.b - amount).max(0.0);
    color
}

pub fn lighten(mut color: Color, amount: f32) -> Color {
    color.r = (color.r + amount).min(1.0);
    color.g = (color.g + amount).min(1.0);
    color.b = (color.b + amount).min(1.0);
    color
}

pub fn layer_1(base_color: Color, is_dark: bool) -> Color {
    if is_dark {
        darken(base_color, 0.015)
    } else {
        darken(base_color, 0.2)
    }
}

pub fn layer_2(base_color: Color, is_dark: bool) -> Color {
    if is_dark {
        darken(base_color, 0.04)
    } else {
        darken(base_color, 0.2)
    }
}

pub fn hover(base_color: Color, is_dark: bool) -> Color {
    if is_dark {
        lighten(base_color, 0.02)
    } else {
        darken(base_color, 0.1)
    }
}

pub fn disabled(base_color: Color, is_dark: bool) -> Color {
    if is_dark {
        lighten(base_color, 0.02)
    } else {
        darken(base_color, 0.2)
    }
}

pub fn muted_light(base_color: Color) -> Color {
    base_color.scale_alpha(0.8)
}

pub fn muted_medium(base_color: Color) -> Color {
    base_color.scale_alpha(0.6)
}

pub fn muted_max(base_color: Color) -> Color {
    base_color.scale_alpha(0.4)
}

pub fn opaque_1(base_color: Color, is_dark: bool) -> Color {
    if is_dark {
        base_color.scale_alpha(0.05)
    } else {
        base_color.scale_alpha(0.75)
    }
}
