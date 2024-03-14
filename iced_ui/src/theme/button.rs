
use super::Theme;

use iced::{theme::palette, Background, widget::button, Border, Color, Vector};


#[derive(Default)]
pub enum Button {
  #[default]
  Primary,
  Secondary,
  Positive,
  Destructive,
  Text,
  MenuButton,
  AccountButton,
}


impl button::StyleSheet for Theme {
  type Style = Button;

  fn active(&self, style: &Self::Style) -> button::Appearance {
      let palette = self.extended_palette();

      let appearance = button::Appearance {
          border: Border::with_radius(2),
          ..button::Appearance::default()
      };

      let from_pair = |pair: palette::Pair| button::Appearance {
          background: Some(pair.color.into()),
          text_color: pair.text,
          ..appearance
      };

      match style {
          Button::Primary => from_pair(palette.primary.strong),
          Button::Secondary => from_pair(palette.secondary.base),
          Button::Positive => from_pair(palette.success.base),
          Button::Destructive => from_pair(palette.danger.base),
          Button::Text | Button::MenuButton | Button::AccountButton => button::Appearance {
              text_color: palette.background.base.text,
              ..appearance
          },
          
      }
  }

  fn hovered(&self, style: &Self::Style) -> button::Appearance {
      let palette = self.extended_palette();

      let active = self.active(style);

      let background = match style {
          Button::Primary => Some(palette.primary.base.color),
          Button::Secondary => Some(palette.background.strong.color),
          Button::Positive => Some(palette.success.strong.color),
          Button::Destructive => Some(palette.danger.strong.color),
          Button::Text => None,
          _ => None,
          
      };

      button::Appearance {
          background: background.map(Background::from),
          ..active
      }
  }

  fn pressed(&self, style: &Self::Style) -> button::Appearance {

      button::Appearance {
          shadow_offset: Vector::default(),
          ..self.active(style)
      }
  }

  fn disabled(&self, style: &Self::Style) -> button::Appearance {

      let active = self.active(style);

      button::Appearance {
          shadow_offset: Vector::default(),
          background: active.background.map(|background| match background {
              Background::Color(color) => Background::Color(Color {
                  a: color.a * 0.5,
                  ..color
              }),
              Background::Gradient(gradient) => {
                  Background::Gradient(gradient.mul_alpha(0.5))
              }
          }),
          text_color: Color {
              a: active.text_color.a * 0.5,
              ..active.text_color
          },
          ..active
      }
  }
  
}