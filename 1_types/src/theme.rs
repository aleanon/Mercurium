use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Theme {
    Light,
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

impl Into<iced::Theme> for Theme {
    fn into(self) -> iced::Theme {
        match self {
            Theme::Light => iced::Theme::Light,
            Theme::Dark => iced::Theme::Dark,
            Theme::Dracula => iced::Theme::Dracula,
            Theme::Nord => iced::Theme::Nord,
            Theme::SolarizedLight => iced::Theme::SolarizedLight,
            Theme::SolarizedDark => iced::Theme::SolarizedDark,
            Theme::GruvboxLight => iced::Theme::GruvboxLight,
            Theme::GruvboxDark => iced::Theme::GruvboxDark,
            Theme::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            Theme::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            Theme::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            Theme::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            Theme::TokyoNight => iced::Theme::TokyoNight,
            Theme::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            Theme::TokyoNightLight => iced::Theme::TokyoNightLight,
            Theme::KanagawaWave => iced::Theme::KanagawaWave,
            Theme::KanagawaDragon => iced::Theme::KanagawaDragon,
            Theme::KanagawaLotus => iced::Theme::KanagawaLotus,
            Theme::Moonfly => iced::Theme::Moonfly,
            Theme::Nightfly => iced::Theme::Nightfly,
            Theme::Oxocarbon => iced::Theme::Oxocarbon,
            Theme::Ferra => iced::Theme::Ferra,
            Theme::Custom => iced::Theme::Dark,
        }
    }
}

impl From<iced::Theme> for Theme {
    fn from(value: iced::Theme) -> Self {
        match value {
            iced::Theme::Light => Theme::Light,
            iced::Theme::Dark => Theme::Dark,
            iced::Theme::Dracula => Theme::Dracula,
            iced::Theme::Nord => Theme::Nord,
            iced::Theme::SolarizedLight => Theme::SolarizedLight,
            iced::Theme::SolarizedDark => Theme::SolarizedDark,
            iced::Theme::GruvboxLight => Theme::GruvboxLight,
            iced::Theme::GruvboxDark => Theme::GruvboxDark,
            iced::Theme::CatppuccinLatte => Theme::CatppuccinLatte,
            iced::Theme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            iced::Theme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            iced::Theme::CatppuccinMocha => Theme::CatppuccinMocha,
            iced::Theme::TokyoNight => Theme::TokyoNight,
            iced::Theme::TokyoNightStorm => Theme::TokyoNightStorm,
            iced::Theme::TokyoNightLight => Theme::TokyoNightLight,
            iced::Theme::KanagawaWave => Theme::KanagawaWave,
            iced::Theme::KanagawaDragon => Theme::KanagawaDragon,
            iced::Theme::KanagawaLotus => Theme::KanagawaLotus,
            iced::Theme::Moonfly => Theme::Moonfly,
            iced::Theme::Nightfly => Theme::Nightfly,
            iced::Theme::Oxocarbon => Theme::Oxocarbon,
            iced::Theme::Ferra => Theme::Ferra,
            iced::Theme::Custom(_) => Theme::Custom,
        }
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Theme::Light => f.write_str("Light"),
            Theme::Dark => f.write_str("Dark"),
            Theme::Dracula => f.write_str("Dracula"),
            Theme::Nord => f.write_str("Nord"),
            Theme::SolarizedLight => f.write_str("Solarized Light"),
            Theme::SolarizedDark => f.write_str("Solarized Dark"),
            Theme::CatppuccinFrappe => f.write_str("CatppuccinFrappe"),
            Theme::CatppuccinLatte => f.write_str("CatppuccinLatte"),
            Theme::CatppuccinMacchiato => f.write_str("CatppuccinMacchiato"),
            Theme::CatppuccinMocha => f.write_str("CatppuccinMocha"),
            Theme::GruvboxLight => f.write_str("Gruvbox Light"),
            Theme::GruvboxDark => f.write_str("Gruvbox Dark"),
            Theme::TokyoNight => f.write_str("Tokyo Night"),
            Theme::TokyoNightStorm => f.write_str("Tokyo Night Storm"),
            Theme::TokyoNightLight => f.write_str("Tokyo Night Light"),
            Theme::KanagawaDragon => f.write_str("KanagawaDragon"),
            Theme::KanagawaLotus => f.write_str("KanagawaLotus"),
            Theme::KanagawaWave => f.write_str("KanagawaWave"),
            Theme::Moonfly => f.write_str("Moonfly"),
            Theme::Nightfly => f.write_str("Nightfly"),
            Theme::Oxocarbon => f.write_str("Oxocarbon"),
            Theme::Ferra => f.write_str("Ferra"),
            Theme::Custom => f.write_str("Custom"),
        }
    }
}
