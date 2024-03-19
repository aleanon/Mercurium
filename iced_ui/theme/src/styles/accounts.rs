use iced::{border::Radius, widget::{button, container, shader::wgpu::core::command::BakedCommands}, Background, Border, Color, Shadow, Theme, Vector};



pub struct AccountButton;

impl button::StyleSheet for AccountButton {
  type Style = Theme;
  fn active(&self, style: &Self::Style) -> button::Appearance {
    let extended_palette = style.extended_palette();
    let mut background_color = extended_palette.background.base.color;
    let shadow_color = extended_palette.background.weak.color;
    let text_color = extended_palette.background.base.text;
    background_color.a -= 0.1;


    button::Appearance { 
      background: Some(iced::Background::Color(background_color)), 
      text_color,
      border: Border { color: Color::TRANSPARENT, width: 1., radius: Radius::from([10.;4])},
      shadow: Shadow { color: shadow_color, offset: Vector::new(2., 2.), blur_radius: 3. },
      shadow_offset: Vector { x: 0., y: 0. }
    }
  }
  
  fn hovered(&self, style: &Self::Style) -> button::Appearance {
    let extended_palette = style.extended_palette();
    let mut background_color = extended_palette.background.weak.color;
    let text_color = extended_palette.background.base.text;
    background_color.a = 0.1;


    button::Appearance { 
      background: Some(iced::Background::Color(background_color)),
      ..self.active(style)
    }      
  }

}

pub struct AccountOverview;


impl AccountOverview {
  pub fn style(theme: &iced::Theme) -> container::Appearance {
    let extended_palette = theme.extended_palette();
    let background_base = extended_palette.background.base;
    let mut background_color = background_base.color;
    let shadow_color = extended_palette.background.weak.color;
    let text_color = background_base.text;
    background_color.a += 0.01;

    // for c in &mut background_color[0..3] {
    //   if let Some(num) = c.checked_add(2) {
    //     *c = num
    //   } else {
    //     *c = 0
    //   }
    // }

    container::Appearance {
        border: iced::Border { color: Color::TRANSPARENT, width: 1., radius: Radius::from([10.;4])},
        shadow: iced::Shadow { color: shadow_color, offset: Vector::new(2., 2.), blur_radius: 3. },
        background: Some(iced::Background::Color(background_color)),
        text_color: Some(text_color),
    }
  }
}

pub struct AssetListItem;

impl AssetListItem {
  pub fn style(theme: &Theme) -> container::Appearance {
    let background = theme.extended_palette().background.base;
    let mut background_color =background.color;
    background_color.a -= 0.01;

    container::Appearance {
      background: Some(iced::Background::Color(background_color)),
      border: Border {radius: Radius::from(0), color: Color::TRANSPARENT, width: 0.},
      shadow: Shadow::default(),
      text_color: None,
    }
  }
}


pub struct AssetListButton;

impl button::StyleSheet for AssetListButton {
  type Style = Theme;

  fn active(&self, style: &Self::Style) -> button::Appearance {
    let background = style.extended_palette().background.base;
    let mut background_color = background.color;
    let text_color = background.text;
    background_color.a -= 0.01;

    button::Appearance {
      background: Some(Background::Color(background_color)),
      text_color: text_color,
      ..Default::default()
    } 
  }  

  fn hovered(&self, style: &Self::Style) -> button::Appearance {
    let mut background_color = style.extended_palette().background.weak.color;
    background_color.a = 0.1;

    button::Appearance {
      background: Some(Background::Color(background_color)),
      ..self.active(style) 
    }
  }

  fn disabled(&self, style: &Self::Style) -> button::Appearance {
      self.active(style)
  }

  fn pressed(&self, style: &Self::Style) -> button::Appearance {
      self.active(style)
  }
}
  
  