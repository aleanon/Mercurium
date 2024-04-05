use iced::{widget::{button, text, text_input::Icon, Scrollable, TextInput}, Element};
use store::Db;
use types::{AccountAddress, Fungibles, NonFungibles, ResourceAddress};

use crate::{message::Message, App};

#[derive(Debug, Clone)]
pub enum AssetTab {
  Tokens,
  NFTs,
}


#[derive(Debug)]
pub struct AddAssets {
  pub tab: AssetTab,
  pub from_account: AccountAddress,
  pub recipient_index: usize,
  pub filter: String,
  pub selected: Vec<(String, ResourceAddress, String)>,
}


impl<'a> AddAssets {
  pub fn new(from_account: AccountAddress, recipient_index: usize) -> Self {
    Self {
      tab: AssetTab::Tokens,      
      from_account,
      recipient_index,
      filter: String::new(),
      selected: Vec::new(), 
    }
  }

//   pub fn view(&self, app: &'a App) -> Element<'a, Message> {
//     let db = app.db.as_ref().unwrap_or_else(|| unreachable!());

//     let search_field = TextInput::new("Search token", &self.filter).line_height(1.5).size(12);

//     let token_button = button(text("Tokens")
//         .size(12)
//         .horizontal_alignment(iced::alignment::Horizontal::Center)
//         .vertical_alignment(iced::alignment::Vertical::Center)
//       )
//       .width(80)
//       .height(30);

//     match self.tab {
//       AssetTab::Tokens => self.tokens_tab(db),
//       AssetTab::NFTs => self.nfts_tab(db),
//     }
//   }

//   fn tokens_tab(&self, db: &Db) -> Scrollable<'a, Message> {
//     let fungibles = db.get_fungibles_by_account(&self.account).unwrap_or(Fungibles::new());
//   }

//   fn nfts_tab(&self,  db: &Db) -> Scrollable<'a, Message> {
//     let non_fungibles = db.get_non_fungibles_by_account(&self.account).unwrap_or(NonFungibles::new());
//   } 
}