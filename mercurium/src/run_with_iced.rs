use std::borrow::Cow;

use font_and_icons::{BOOTSTRAP_FONT_BYTES, images::WINDOW_LOGO};

pub fn run() -> Result<(), deps::iced::Error> {
    use deps::iced::{Settings, Size, window};
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
    deps::iced::application(App::new, App::update, App::view)
        .title(types::consts::APPLICATION_NAME)
        .settings(settings)
        .theme(App::theme)
        .style(App::style)
        .window(window_settings)
        .run()?;

    #[cfg(feature = "reload")]
    {
        let reloader_settings = deps::hot_ice::ReloaderSettings {
            compile_in_reloader: true,
            feature: Some("reload".to_string()),
            ..Default::default()
        };

        deps::hot_ice::application(App::new, App::update, App::view)
            .reloader_settings(reloader_settings)
            .title(App::title)
            .settings(settings)
            .window(window_settings)
            .theme(App::theme)
            .style(App::style)
            .run()?;
    }

    Ok(())
}
