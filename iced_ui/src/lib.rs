mod app;
mod common;
mod error;
mod external_task_response;
mod external_tasks;
mod initial;
mod locked;
mod subscription;
mod unlocked;

use std::borrow::Cow;

pub use app::App;
use font_and_icons::{images::WINDOW_LOGO, BOOTSTRAP_FONT_BYTES};
use iced::{
    application::application,
    window::{self},
    Settings,
};

pub fn run() -> Result<(), iced::Error> {
    let icon = window::icon::from_file_data(
        WINDOW_LOGO,
        Some(iced::advanced::graphics::image::image_rs::ImageFormat::Png),
    )
    .unwrap();

    let mut settings = Settings {
        antialiasing: false,
        id: Some(String::from("ravault")),
        ..Default::default()
    };
    settings.fonts.push(Cow::Borrowed(BOOTSTRAP_FONT_BYTES));

    application(types::consts::APPLICATION_NAME, App::update, App::view)
        .settings(settings)
        .window(window::Settings {
            icon: Some(icon),
            platform_specific: window::settings::PlatformSpecific {
                skip_taskbar: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .run_with(|| App::new())?;

    Ok(())
}
