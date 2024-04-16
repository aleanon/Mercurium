mod app;
mod icons;
mod message;
mod subscription;
mod view;
// mod styles;
//mod theme;

use std::borrow::Cow;

pub use app::App;
use iced::{
    window::{self},
    Application, Settings,
};
const WINDOW_ICON: &'static [u8] = include_bytes!("../../icons/ravault_window_icon.png");

pub fn run() -> Result<(), iced::Error> {
    let icon = window::icon::from_file_data(
        WINDOW_ICON,
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
    settings
        .fonts
        .push(Cow::Borrowed(iced_aw::BOOTSTRAP_FONT_BYTES));

    App::run(settings)?;
    Ok(())
}
