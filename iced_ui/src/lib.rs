#![feature(let_chains)]

use std::borrow::Cow;

// use deps::*;

pub mod app;
mod common;
mod common_elements;
mod components;
mod error;
mod initial;
mod locked;
mod styles;
mod unlocked;

pub use app::App;
// use font_and_icons::{images::WINDOW_LOGO, BOOTSTRAP_FONT_BYTES};
pub use deps::iced::Error;

// fn main() -> Result<(), iced::Error> {
//     dioxus_devtools::connect_subsecond();
//     subsecond::call(|| {
//         use deps::iced::{application, window, Settings, Size};
//         // use iced_ui::App;

//         let icon = window::icon::from_file_data(WINDOW_LOGO, None).unwrap();

//         let mut settings = Settings {
//             antialiasing: true,
//             ..Default::default()
//         };
//         settings.fonts.push(Cow::Borrowed(BOOTSTRAP_FONT_BYTES));

//         let window_settings = window::Settings {
//             min_size: Some(Size {
//                 height: 800.,
//                 width: 1000.,
//             }),
//             icon: Some(icon),
//             ..Default::default()
//         };

//         #[cfg(not(feature = "reload"))]
//         application(App::new, App::update, App::view)
//             .title(types::consts::APPLICATION_NAME)
//             .settings(settings)
//             .theme(|app| app.preferences.theme.into())
//             .window(window_settings)
//             .run()?;

//         #[cfg(feature = "reload")]
//         deps::hot_ice::hot_application("target/reload", App::new, App::update, App::view)
//             .title(types::consts::APPLICATION_NAME)
//             .settings(settings)
//             .theme(|app| app.preferences.theme.into())
//             .window(window_settings)
//             .run()
//             .unwrap();

//         Ok(())
//     })
// }
