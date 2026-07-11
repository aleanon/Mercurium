use deps::*;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    #[default]
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
    Custom,
}

impl Theme {
    pub fn as_str(&self) -> &str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
            Theme::Dracula => "Dracula",
            Theme::Nord => "Nord",
            Theme::SolarizedLight => "Solarized Light",
            Theme::SolarizedDark => "Solarized Dark",
            Theme::GruvboxLight => "Gruvbox Light",
            Theme::GruvboxDark => "Gruvbox Dark",
            Theme::CatppuccinLatte => "Catppuccin Latte",
            Theme::CatppuccinFrappe => "Catppuccin Frappe",
            Theme::CatppuccinMacchiato => "Catppuccin Macchiato",
            Theme::CatppuccinMocha => "Catppuccin Mocha",
            Theme::TokyoNight => "Tokyo Night",
            Theme::TokyoNightStorm => "Tokyo Night Storm",
            Theme::TokyoNightLight => "Tokyo Night Light",
            Theme::KanagawaWave => "Kanagawa Wave",
            Theme::KanagawaDragon => "Kanagawa Dragon",
            Theme::KanagawaLotus => "Kanagawa Lotus",
            Theme::Moonfly => "Moonfly",
            Theme::Nightfly => "Nightfly",
            Theme::Oxocarbon => "Oxocarbon",
            Theme::Ferra => "Ferra",
            Theme::Custom => "Custom",
        }
    }
}

// Conversion to/from iced::Theme lives in the iced_ui crate (see
// `iced_ui::app::to_iced_theme`), so this domain crate carries no dependency on
// the GUI framework. The orphan rule requires the conversion to live in a crate
// that owns one of the types, hence a free function there rather than a From impl.

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark() {
        assert_eq!(Theme::default().as_str(), "Dark");
    }

    #[test]
    fn as_str_and_display_agree() {
        assert_eq!(Theme::SolarizedLight.as_str(), "Solarized Light");
        assert_eq!(format!("{}", Theme::Nord), "Nord");
    }

    #[test]
    fn serde_roundtrip() {
        let json = deps::serde_json::to_string(&Theme::GruvboxDark).unwrap();
        let back: Theme = deps::serde_json::from_str(&json).unwrap();
        assert_eq!(back.as_str(), "Gruvbox Dark");
    }
}
