use iced::{border::Radius, widget::rule::{self, FillMode}, Theme};



pub struct TextInputRule;

impl TextInputRule {
  pub fn style(theme: &Theme) -> rule::Appearance {

    rule::Appearance {
      radius: Radius::from([0.,0.,10.,10.]),
      fill_mode: FillMode::Full,
      width: 4,
      color: theme.extended_palette().primary.base.color
    }
  }
}