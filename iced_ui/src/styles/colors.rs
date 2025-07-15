use deps::iced::{self, color, Color, Theme};

use crate::styles::colors::dark::BACKGROUND_PRIMARY;

pub fn background_primary(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::BACKGROUND_PRIMARY,
        Theme::Dark => dark::BACKGROUND_PRIMARY,
        _ => dark::BACKGROUND_PRIMARY,
    }
}

pub fn background_secondary(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::BACKGROUND_SECONDARY,
        Theme::Dark => dark::BACKGROUND_SECONDARY,
        _ => dark::BACKGROUND_SECONDARY,
    }
}

pub fn background_elevated(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::BACKGROUND_ELEVATED,
        Theme::Dark => dark::BACKGROUND_ELEVATED,
        _ => dark::BACKGROUND_ELEVATED,
    }
}

pub fn background_card(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::BACKGROUND_CARD,
        Theme::Dark => dark::BACKGROUND_CARD,
        _ => dark::BACKGROUND_CARD,
    }
}

pub fn background_disabled(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::BACKGROUND_DISABLED,
        Theme::Dark => dark::BACKGROUND_DISABLED,
        _ => dark::BACKGROUND_DISABLED,
    }
}

// Text Colors
pub fn text_primary(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::TEXT_PRIMARY,
        Theme::Dark => dark::TEXT_PRIMARY,
        _ => dark::TEXT_PRIMARY,
    }
}

pub fn text_secondary(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::TEXT_SECONDARY,
        Theme::Dark => dark::TEXT_SECONDARY,
        _ => dark::TEXT_SECONDARY,
    }
}

pub fn text_muted(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::TEXT_MUTED,
        Theme::Dark => dark::TEXT_MUTED,
        _ => dark::TEXT_MUTED,
    }
}

pub fn text_disabled(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::TEXT_DISABLED,
        Theme::Dark => dark::TEXT_DISABLED,
        _ => dark::TEXT_DISABLED,
    }
}

// Accent Colors
pub fn accent_primary_base(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::ACCENT_PRIMARY_BASE,
        Theme::Dark => dark::ACCENT_PRIMARY_BASE,
        _ => dark::ACCENT_PRIMARY_BASE,
    }
}

pub fn accent_primary_weak(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::ACCENT_PRIMARY_WEAK,
        Theme::Dark => dark::ACCENT_PRIMARY_WEAK,
        _ => dark::ACCENT_PRIMARY_WEAK,
    }
}

pub fn accent_primary_strong(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::ACCENT_PRIMARY_STRONG,
        Theme::Dark => dark::ACCENT_PRIMARY_STRONG,
        _ => dark::ACCENT_PRIMARY_STRONG,
    }
}

pub fn accent_success(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::ACCENT_SUCCESS,
        Theme::Dark => dark::ACCENT_SUCCESS,
        _ => dark::ACCENT_SUCCESS,
    }
}

pub fn accent_warning(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::ACCENT_WARNING,
        Theme::Dark => dark::ACCENT_WARNING,
        _ => dark::ACCENT_WARNING,
    }
}

pub fn accent_error(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::ACCENT_ERROR,
        Theme::Dark => dark::ACCENT_ERROR,
        _ => dark::ACCENT_ERROR,
    }
}

pub fn accent_info(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::ACCENT_INFO,
        Theme::Dark => dark::ACCENT_INFO,
        _ => dark::ACCENT_INFO,
    }
}

// Interactive States
pub fn state_hover(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::STATE_HOVER,
        Theme::Dark => dark::STATE_HOVER,
        _ => dark::STATE_HOVER,
    }
}

pub fn state_active(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::STATE_ACTIVE,
        Theme::Dark => dark::STATE_ACTIVE,
        _ => dark::STATE_ACTIVE,
    }
}

pub fn state_focus(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::STATE_FOCUS,
        Theme::Dark => dark::STATE_FOCUS,
        _ => dark::STATE_FOCUS,
    }
}

pub fn state_selection(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::STATE_SELECTION,
        Theme::Dark => dark::STATE_SELECTION,
        _ => dark::STATE_SELECTION,
    }
}

pub fn state_disabled(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::STATE_DISABLED,
        Theme::Dark => dark::STATE_DISABLED,
        _ => dark::STATE_DISABLED,
    }
}

// Borders
pub fn border_subtle(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::BORDER_SUBTLE,
        Theme::Dark => dark::BORDER_SUBTLE,
        _ => dark::BORDER_SUBTLE,
    }
}

pub fn border_default(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::BORDER_DEFAULT,
        Theme::Dark => dark::BORDER_DEFAULT,
        _ => dark::BORDER_DEFAULT,
    }
}

pub fn border_strong(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::BORDER_STRONG,
        Theme::Dark => dark::BORDER_STRONG,
        _ => dark::BORDER_STRONG,
    }
}

pub fn shadow_base(theme: &Theme) -> Color {
    match theme {
        Theme::Light => light::SHADOW_BASE,
        Theme::Dark => dark::SHADOW_BASE,
        _ => dark::SHADOW_BASE,
    }
}

pub mod dark {
    use super::*;
    // Background Colors
    pub const BACKGROUND_PRIMARY: Color = color!(0x191919);
    pub const BACKGROUND_SECONDARY: Color = color!(0x212121);
    pub const BACKGROUND_ELEVATED: Color = color!(0x303030);
    pub const BACKGROUND_CARD: Color = color!(0x282828);
    pub const BACKGROUND_DISABLED: Color = color!(0x303030);

    // Text Colors
    pub const TEXT_PRIMARY: Color = color!(0xffffff);
    pub const TEXT_SECONDARY: Color = color!(0xb8b8b8);
    pub const TEXT_MUTED: Color = color!(0x7a7a7a);
    pub const TEXT_DISABLED: Color = color!(0x474747);

    // Accent Colors
    pub const ACCENT_PRIMARY_WEAK: Color = color!(0x3780ef);
    pub const ACCENT_PRIMARY_BASE: Color = color!(0x4285f4);
    pub const ACCENT_PRIMARY_STRONG: Color = color!(0x4790f9);
    pub const ACCENT_SUCCESS: Color = color!(0x34a853);
    pub const ACCENT_WARNING: Color = color!(0xff9800);
    pub const ACCENT_ERROR: Color = color!(0xea4335);
    pub const ACCENT_INFO: Color = color!(0x9c27b0);

    // Interactive States
    pub const STATE_HOVER: Color = color!(0x383838);
    pub const STATE_ACTIVE: Color = color!(0x454545);
    pub const STATE_FOCUS: Color = color!(0x4285f4);
    pub const STATE_SELECTION: Color = color!(0x4285f4);
    pub const STATE_DISABLED: Color = color!(0x474747);

    // Borders
    pub const BORDER_SUBTLE: Color = color!(0x343434);
    pub const BORDER_DEFAULT: Color = color!(0x454545);
    pub const BORDER_STRONG: Color = color!(0x575757);

    // Shadows
    pub const SHADOW_BASE: Color = Color::TRANSPARENT;
}

pub mod light {
    use super::*;
    // Background Colors
    pub const BACKGROUND_PRIMARY: Color = color!(0xffffff);
    pub const BACKGROUND_SECONDARY: Color = color!(0xf8f9fa);
    pub const BACKGROUND_ELEVATED: Color = color!(0xffffff);
    pub const BACKGROUND_CARD: Color = color!(0xffffff);
    pub const BACKGROUND_DISABLED: Color = color!(0xf5f5f5);
    // Text Colors
    pub const TEXT_PRIMARY: Color = color!(0x1a1a1a);
    pub const TEXT_SECONDARY: Color = color!(0x5f6368);
    pub const TEXT_MUTED: Color = color!(0x9aa0a6);
    pub const TEXT_DISABLED: Color = color!(0xbdc1c6);
    // Accent Colors
    pub const ACCENT_PRIMARY_WEAK: Color = color!(0x1568e3);
    pub const ACCENT_PRIMARY_BASE: Color = color!(0x1a73e8);
    pub const ACCENT_PRIMARY_STRONG: Color = color!(0x1f78ed);
    pub const ACCENT_SUCCESS: Color = color!(0x137333);
    pub const ACCENT_WARNING: Color = color!(0xf29900);
    pub const ACCENT_ERROR: Color = color!(0xd93025);
    pub const ACCENT_INFO: Color = color!(0x8e24aa);
    // Interactive States
    pub const STATE_HOVER: Color = color!(0xf1f3f4);
    pub const STATE_ACTIVE: Color = color!(0xe8eaed);
    pub const STATE_FOCUS: Color = color!(0x1a73e8);
    pub const STATE_SELECTION: Color = color!(0x1a73e8);
    pub const STATE_DISABLED: Color = color!(0xbdc1c6);
    // Borders
    pub const BORDER_SUBTLE: Color = color!(0xf1f3f4);
    pub const BORDER_DEFAULT: Color = color!(0xe8eaed);
    pub const BORDER_STRONG: Color = color!(0xdadce0);

    // Shadows
    pub const SHADOW_BASE: Color = color!(0x101010);
}
