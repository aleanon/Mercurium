use iced::{border::Radius, widget::text_input::{self, StyleSheet}, Border, Color, Theme};




pub struct TextInput;

impl text_input::StyleSheet for TextInput {
  type Style = Theme;

  fn active(&self, style: &Self::Style) -> text_input::Appearance {
    let extended_palette = style.extended_palette();
    let background = extended_palette.background.base.color;
    let border = Border {width: 0., color: Color::TRANSPARENT, radius: Radius::from([5.,5.,0.,0.])};

     text_input::Appearance { 
      background: iced::Background::Color(background), 
      border, 
      icon_color: Color::TRANSPARENT 
    } 
  }

  fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
      self.active(style)
  }

  fn disabled_color(&self, style: &Self::Style) -> Color {
      style.extended_palette().background.weak.color
  }

  fn focused(&self, style: &Self::Style) -> text_input::Appearance {
      self.active(style)
  }

  fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
      self.active(style)
  }

  fn value_color(&self, style: &Self::Style) -> Color {
      style.extended_palette().background.base.text
  }

  fn placeholder_color(&self, style: &Self::Style) -> Color {
      style.extended_palette().background.weak.text
  }

  fn selection_color(&self, style: &Self::Style) -> Color {
      style.extended_palette().background.strong.color
  }
}