use std::borrow::Cow;

use font_and_icons::{BOOTSTRAP_FONT_BYTES, images::WINDOW_LOGO};

// #[cfg(not(feature="reload"))]
use iced_ui::app;

// #[cfg(feature="reload")]
// use app::*;

// #[cfg(feature = "reload")]
// #[hot_lib_reloader::hot_module(dylib = "iced_ui", lib_dir = "target/reload")]
// mod app {
//     use iced_ui::app::*;
//     hot_functions_from_file!("iced_ui/src/app.rs", ignore_no_mangle = true);
// }

pub fn run() -> Result<(), deps::iced::Error> {
    use deps::iced::{Settings, Size, application, window};
    use iced_ui::App;

    let icon = window::icon::from_file_data(WINDOW_LOGO, None).unwrap();

    let mut settings = Settings {
        antialiasing: true,
        ..Default::default()
    };
    settings.fonts.push(Cow::Borrowed(BOOTSTRAP_FONT_BYTES));

    let window_settings = window::Settings {
        min_size: Some(Size {
            height: 400.,
            width: 300.,
        }),
        icon: Some(icon),
        ..Default::default()
    };

    #[cfg(not(feature = "reload"))]
    application(App::new, App::update, App::view)
        .title(types::consts::APPLICATION_NAME)
        .settings(settings)
        .theme(|app| app.preferences.theme.into())
        .window(window_settings)
        .run()?;

    #[cfg(feature = "reload")]
    deps::hot_ice::hot_application("target/reload", App::new, App::update, App::view)
        .title(types::consts::APPLICATION_NAME)
        .settings(settings)
        .theme(|app| app.preferences.theme.into())
        .window(window_settings)
        .run()
        .unwrap();

    Ok(())
}
