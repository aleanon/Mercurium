
use std::fmt;

use iced::theme::{palette, Palette};


#[derive(Debug, Default)]
pub enum Theme {
  #[default]
  Dark,
}

impl Theme {

  pub const ALL: &'static [Self] = &[
          Self::Dark,
      ];


      /// Returns the [`Palette`] of the [`Theme`].
      pub fn palette(&self) -> Palette {
          match self {
              Self::Dark => Palette::DARK,
          }
      }

      /// Returns the [`palette::Extended`] of the [`Theme`].
      pub fn extended_palette(&self) -> &palette::Extended {
          match self {
              Self::Dark => &palette::EXTENDED_DARK,
          }
      }
  }

  impl fmt::Display for Theme {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          match self {
              Self::Dark => write!(f, "Dark"),
          }
    }
}


