use iced::border::Radius;
use iced::gradient::{ColorStop, Linear};
use iced::widget::container;
use iced::widget::shader::wgpu::naga::back;
use iced::{color, theme, Background, Border, Gradient, Radians, Vector};
use iced::{widget::button, Color, Shadow, Theme};

#[derive(Default)]
pub struct MenuButton;

impl button::StyleSheet for MenuButton {
  type Style = Theme;

  fn active(&self, style: &Self::Style) -> button::Appearance {
    let text_color = style.extended_palette().background.base.text;

    button::Appearance { 
      shadow_offset: iced::Vector { x: 0., y: 0. }, 
      background: None, 
      text_color, 
      border: Border {color: Color::TRANSPARENT, radius: Radius::from(10.), width: 0.}, 
      shadow: Shadow { color: Color::TRANSPARENT, offset: Vector::ZERO, blur_radius: 0. } 
    }
  }

  fn hovered(&self, style: &Self::Style) -> button::Appearance {
    let text_color = style.extended_palette().background.strong.text;

    button::Appearance {
      text_color,
      ..self.active(style)
    }
  }

  fn disabled(&self, style: &Self::Style) -> button::Appearance {
      let text_color = style.extended_palette().background.weak.text;

      button::Appearance {
        text_color,
        ..self.active(style)
      }
  }

  fn pressed(&self, style: &Self::Style) -> button::Appearance {
      self.hovered(style)
  }
}

pub struct SelectedMenuButton; 

impl button::StyleSheet for SelectedMenuButton {
  type Style = Theme;

  fn active(&self, style: &Self::Style) -> button::Appearance {
    let mut background_color = style.extended_palette().background.weak.color;
    background_color.a = 0.1;


    button::Appearance { 
      background: Some(Background::Color(background_color)),
      ..MenuButton.hovered(style)
    }
  }

  fn hovered(&self, style: &Self::Style) -> button::Appearance {
      self.active(style)
  }

  fn pressed(&self, style: &Self::Style) -> button::Appearance {
      self.active(style)
  }

}





pub struct MenuContainer;


impl MenuContainer {
  const BACKGROUND_ALPHA_STEP: f32 = 0.001;
  const MENU_ALPHA: f32 = 0.1;
  const STOPS_LEN: usize = 8;

  pub fn style(theme: &Theme) -> container::Appearance {
    let mut background_color = theme.extended_palette().background.base.color;
    
    let mut stops:[Option<ColorStop>; Self::STOPS_LEN] = [None; Self::STOPS_LEN];
    let mut current_alpha = background_color.a;
    let mut current_offset = 0.;

    
    
    for i in 0..Self::STOPS_LEN {
      let mut color = background_color.clone();
      color.a = current_alpha;
      stops[i] = Some(ColorStop {color, offset: current_offset});
      current_offset += 0.12;
      current_alpha -= Self::BACKGROUND_ALPHA_STEP
    }
    
    let background = Some(Background::Gradient(Gradient::Linear(Linear {
      angle: Radians(1.570796),
      stops
    })));

    container::Appearance { 
      text_color: None, 
      background,
      border: Border::default(), 
      shadow: Shadow::default() 
    }
  }
}
