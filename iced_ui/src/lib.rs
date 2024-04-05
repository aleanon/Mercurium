mod icons;
mod app;
mod message;
mod view;
mod subscription;
// mod styles;
//mod theme;

pub use app::App;
use iced::{window::{self, Icon}, Application, Settings};
const WINDOW_ICON: &'static [u8] = include_bytes!("../../icons/ravault_window_icon.png");


pub fn run() -> Result<(), iced::Error> {
    let icon = window::icon::from_file_data(WINDOW_ICON, Some(iced::advanced::graphics::image::image_rs::ImageFormat::Png)).unwrap();

    let settings: iced::Settings<()> = Settings {
        flags: (),
        antialiasing: false,
        window: iced::window::Settings {
            icon: Some(icon),
            min_size: Some(iced::Size { width: 1000., height: 700. }),
            position: iced::window::Position::Centered,
            ..Default::default()
        },
        default_font: Default::default(),
        id: Some(String::from("ravault")),
        ..Default::default()
    };
    App::run(settings)?;
    Ok(())
}
