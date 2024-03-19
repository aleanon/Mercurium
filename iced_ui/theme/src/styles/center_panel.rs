use iced::{widget::container, Theme};



pub struct CenterPanel;

impl CenterPanel {
  pub fn style(theme:&Theme) -> container::Appearance {
    let mut background = theme.extended_palette().background.base.color;
    background.a -= 0.004;

    container::Appearance {
      background: Some(iced::Background::Color(background)),
      ..Default::default()
    }
  }
}