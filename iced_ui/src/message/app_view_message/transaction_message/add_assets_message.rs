use std::str::FromStr;

use debug_print::{debug_print, debug_println};
use iced::Command;
use types::ResourceAddress;

use crate::{message::{app_view_message::AppViewMessage, Message}, view::app_view::{transaction_view::{self, add_assets::{AddAssets, AssetTab}, Recipient, TransactionView, View}, ActiveTab, AppView}, App};

use super::TransactionMessage;


#[derive(Debug, Clone)]
pub enum AddAssetsMessage {
  SetTab(AssetTab),
  FilterInput(String),
  InputAmount(ResourceAddress, String, String),
  SelectAsset(ResourceAddress, String),
  UnselectAsset(ResourceAddress),
  SubmitAssets,
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
  pub fn process(self, transaction_view: &'a mut TransactionView) -> Command<Message> {
    let view = &mut transaction_view.view;
    if let View::ChooseResource(ref mut add_assets) = view {
      let command = Command::none();

      match self {
        Self::SetTab(tab) => add_assets.tab = tab,
        Self::FilterInput(input) => Self::update_filter(input, add_assets),
        Self::InputAmount(asset_address, symbol, amount) => Self::change_asset_amount(asset_address, symbol, amount, add_assets),
        Self::SelectAsset(asset_address, symbol) => Self::add_selected_asset(asset_address, symbol, add_assets),
        Self::UnselectAsset(asset_address) => Self::remove_selected_asset(asset_address, add_assets),
        Self::SubmitAssets => {
          Self::submit_selected_assets(add_assets, &mut transaction_view.recipients);
          transaction_view.view = View::Transaction;
        }
      }

      command

    } else {
      unreachable!("{}:{} Invalid state", module_path!(), line!())
    }
  }

  fn update_filter(mut input: String, add_assets: &mut AddAssets) {
    input.make_ascii_lowercase();
    add_assets.filter = input
  }

  fn change_asset_amount(asset: ResourceAddress, symbol: String, new_amount: String, add_assets: &mut AddAssets) {
    if new_amount.parse::<f32>().is_ok() {
      if let Some((_, amount)) = add_assets.selected.get_mut(&asset) {
        *amount = new_amount
      } else {
        add_assets.selected.insert(asset, (symbol, new_amount));
      }
    } else if new_amount.is_empty() {
        add_assets.selected.remove(&asset);
    }
  }

  fn add_selected_asset(asset_address: ResourceAddress, symbol: String, add_assets: &mut AddAssets) {
    add_assets.selected.insert(asset_address, (symbol, "".to_string()));
  }

  fn remove_selected_asset(asset_address: ResourceAddress, add_assets: &mut AddAssets) {
    add_assets.selected.remove_entry(&asset_address);
  }

  fn submit_selected_assets(add_assets: &mut AddAssets, recipients: &mut Vec<Recipient>) {
    for (resource_address, (symbol, amount)) in add_assets.selected.drain() {
      if let Some((_, old_amount)) = recipients[add_assets.recipient_index].resources.get_mut(&resource_address) {
        *old_amount = amount 
      } else {
        recipients[add_assets.recipient_index].resources.insert(resource_address, (symbol, amount));
      }
    }
  }
}