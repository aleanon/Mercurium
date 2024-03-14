mod app;
mod message;
mod view;
mod subscription;
mod styles;
mod theme;

pub use app::App;
use iced::{Application, Settings};



pub fn run() -> Result<(), iced::Error> {
  let settings: iced::Settings<()> = Settings {
        flags: (),
        antialiasing: false,
        window: iced::window::Settings {
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
