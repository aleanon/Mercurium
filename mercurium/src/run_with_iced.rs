use std::borrow::Cow;

use font_and_icons::{BOOTSTRAP_FONT_BYTES, images::WINDOW_LOGO};

use iced_ui::app;

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
        .style(App::style)
        .window(window_settings)
        .run()?;

    #[cfg(feature = "reload")]
    deps::hot_ice::hot_application(
        "target/reload",
        App::inner_new,
        App::inner_update,
        App::inner_view,
    )
    .title(types::consts::APPLICATION_NAME)
    .settings(settings)
    .theme(|app| app.preferences.theme.into())
    .window(window_settings)
    .style(App::style)
    .run()
    .unwrap();

    Ok(())
}
