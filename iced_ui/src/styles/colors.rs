use deps::iced::{self, color, Color};

pub mod dark {
    use super::*;
    // Background Colors
    pub const BACKGROUND_PRIMARY: Color = color!(0x191919);
    pub const BACKGROUND_SECONDARY: Color = color!(0x212121);
    pub const BACKGROUND_ELEVATED: Color = color!(0x303030);
    pub const BACKGROUND_CARD: Color = color!(0x282828);

    // Text Colors
    pub const TEXT_PRIMARY: Color = color!(0xffffff);
    pub const TEXT_SECONDARY: Color = color!(0xb8b8b8);
    pub const TEXT_MUTED: Color = color!(0x7a7a7a);
    pub const TEXT_DISABLED: Color = color!(0x474747);

    // Accent Colors
    pub const ACCENT_PRIMARY: Color = color!(0x4285f4);
    pub const ACCENT_SUCCESS: Color = color!(0x34a853);
    pub const ACCENT_WARNING: Color = color!(0xff9800);
    pub const ACCENT_ERROR: Color = color!(0xea4335);
    pub const ACCENT_INFO: Color = color!(0x9c27b0);

    // Interactive States
    pub const STATE_HOVER: Color = color!(0x383838);
    pub const STATE_ACTIVE: Color = color!(0x454545);
    pub const STATE_FOCUS: Color = color!(0x4285f4);
    pub const STATE_SELECTION: Color = color!(0x4285f4);

    // Borders
    pub const BORDER_SUBTLE: Color = color!(0x343434);
    pub const BORDER_DEFAULT: Color = color!(0x454545);
    pub const BORDER_STRONG: Color = color!(0x575757);
}
