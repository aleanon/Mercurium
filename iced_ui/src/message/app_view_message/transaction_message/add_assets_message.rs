use std::{collections::HashMap, str::FromStr};

use debug_print::{debug_print, debug_println};
use iced::Command;
use types::{AccountAddress, Fungible, Fungibles, ResourceAddress};

use crate::{app::AppData, message::{app_view_message::AppViewMessage, Message}, view::app_view::{transaction_view::{self, add_assets::{AddAssets, AssetTab}, Recipient, TransactionView, View}, ActiveTab, AppView}, App};

use super::TransactionMessage;


#[derive(Debug, Clone)]
pub enum AddAssetsMessage {
  SetTab(AssetTab),
  FilterInput(String),
  InputAmount(ResourceAddress, String, String),
  InputMaxSelected,
  SelectAllTokens,
  SelectAsset(ResourceAddress, String),
  UnselectAllTokens,
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
  pub fn process(self, transaction_view: &'a mut TransactionView, appdata: &'a mut AppData) -> Command<Message> {
    let view = &mut transaction_view.view;
    if let View::ChooseResource(ref mut add_assets) = view {
      let command = Command::none();

      match self {
        Self::SetTab(tab) => add_assets.tab = tab,
        Self::FilterInput(input) => Self::update_filter(input, add_assets),
        Self::InputAmount(asset_address, symbol, amount) => Self::change_asset_amount(asset_address, symbol, amount, add_assets),
        Self::InputMaxSelected => Self::set_max_amount_for_selected(add_assets, appdata),
        Self::SelectAllTokens => Self::select_all_tokens(add_assets, appdata),
        Self::SelectAsset(asset_address, symbol) => Self::add_selected_asset(asset_address, symbol, add_assets),
        Self::UnselectAllTokens => {add_assets.selected.clear(); add_assets.select_all = false},
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

  fn set_max_amount_for_selected(add_assets: &mut AddAssets, appdata: &mut AppData) {
    let fungibles = appdata.db.get_fungibles_by_account(&add_assets.from_account).unwrap_or(Fungibles::new())
      .into_iter().map(|fungible| (fungible.address.clone(), fungible)).collect::<HashMap<ResourceAddress, Fungible>>();

    for (resource_address, (symbol, amount)) in &mut add_assets.selected {
      let fungible = fungibles.get(&resource_address).unwrap_or_else(|| unreachable!("Selected asset does not exist"));

      *amount = fungible.amount.to_string();
    }
  }

  fn select_all_tokens(add_assets: &mut AddAssets, appdata: &mut AppData) {
    let fungibles = appdata.db.get_fungibles_by_account(&add_assets.from_account).unwrap_or(Fungibles::new());

    add_assets.selected.reserve(fungibles.len() - add_assets.selected.len());

    for fungible in fungibles {
      add_assets.selected.entry(fungible.address)
        .or_insert((fungible.symbol, "".to_owned()));
    }

    add_assets.select_all = true
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
    add_assets.select_all = false;
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