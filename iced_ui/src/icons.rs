use iced::widget::image::Handle;


const NO_IMAGE_ICON: &'static [u8] = include_bytes!("../../icons/icons8-image-96.png");
const THEME_ICON:&'static [u8] = include_bytes!("../../icons/theme.png");
const ACCOUNTS_ICON:&'static [u8] = include_bytes!("../../icons/bank-account.png");
const TRANSACTION_ICON:&'static [u8] = include_bytes!("../../icons/transfer.png");
const MENU_LOGO:&'static [u8] = include_bytes!("../../icons/menu_logo.png");


#[derive(Debug)]
pub struct Icons {
  pub no_image: Handle,
  pub theme: Handle,
  pub accounts: Handle,
  pub transaction: Handle,
  pub menu_logo: Handle,
}
pub const TRANSACTION: char = '\u{F12B}';


impl Icons {
  pub const TRANSACTION: char = '\u{F12B}';

  pub fn new() -> Self {
    Self {
      no_image: Handle::from_memory(NO_IMAGE_ICON),
      theme: Handle::from_memory(THEME_ICON),
      accounts: Handle::from_memory(ACCOUNTS_ICON),
      transaction: Handle::from_memory(TRANSACTION_ICON),
      menu_logo: Handle::from_memory(MENU_LOGO),
    }
  }
}