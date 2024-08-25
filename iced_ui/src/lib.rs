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
    advanced::Application,
    window::{self},
    Settings,
};

pub fn run() -> Result<(), iced::Error> {
    let icon = window::icon::from_file_data(
        WINDOW_LOGO,
        Some(iced::advanced::graphics::image::image_rs::ImageFormat::Png),
    )
    .unwrap();

    let mut settings: iced::Settings<()> = Settings {
        flags: (),
        antialiasing: false,
        window: iced::window::Settings {
            icon: Some(icon),
            min_size: Some(iced::Size {
                width: 1000.,
                height: 700.,
            }),
            position: iced::window::Position::Centered,
            ..Default::default()
        },
        id: Some(String::from("ravault")),
        ..Default::default()
    };
    settings.fonts.push(Cow::Borrowed(BOOTSTRAP_FONT_BYTES));

    App::run(settings)?;
    Ok(())
}
