use iced::Command;

use crate::{message::{app_view_message::AppViewMessage, Message}, view::app_view::{transaction_view::{add_assets::{AddAssets, AssetTab}, TransactionView, View}, ActiveTab, AppView}, App};

use super::TransactionMessage;


#[derive(Debug, Clone)]
pub enum AddAssetsMessage {
  SetTab(AssetTab),
  FilterInput(String),
}

impl Into<Message> for AddAssetsMessage {
  fn into(self) -> Message {
      Message::AppView(
        AppViewMessage::TransferMessage(
          TransactionMessage::AddAssetsMessage(self)
        )
      )
  }
}

impl<'a> AddAssetsMessage {
  pub fn process(self, parent: &'a mut TransactionView) -> Command<Message> {
    if let View::ChooseResource(ref mut add_assets) = parent.view {
      let command = Command::none();

      match self {
        Self::SetTab(tab) => add_assets.tab = tab,
        Self::FilterInput(input) => add_assets.filter = input,
      }

      command

    } else {
      unreachable!("{}:{} Invalid state", module_path!(), line!())
    }
  }

}