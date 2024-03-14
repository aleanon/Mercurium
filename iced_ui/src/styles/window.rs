use iced::{widget::container, Color, Theme};



pub struct MainWindow;


impl MainWindow {
  pub fn style(theme: &Theme) -> container::Appearance {
    let background = theme.extended_palette().background.base.color.inverse();

    container::Appearance { 
      background: Some(iced::Background::Color(background)),
      ..Default::default()
    }
  }
}