use std::collections::{BTreeSet, HashMap};

use iced::Command;
use types::{assets::FungibleAsset, Decimal, Fungible, Fungibles, RadixDecimal, ResourceAddress};

use crate::{
    app::AppData,
    message::{app_view_message::AppViewMessage, Message},
    view::app_view::transaction_view::{
        add_assets::{AddAssets, AssetTab},
        Recipient, TransactionView, View,
    },
};

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
        Message::AppView(AppViewMessage::TransferMessage(
            TransactionMessage::AddAssetsMessage(self),
        ))
    }
}

impl<'a> AddAssetsMessage {
    pub fn process(
        self,
        transaction_view: &'a mut TransactionView,
        appdata: &'a mut AppData,
    ) -> Command<Message> {
        let view = &mut transaction_view.view;
        if let View::ChooseResource(ref mut add_assets) = view {
            let command = Command::none();

            match self {
                Self::SetTab(tab) => add_assets.tab = tab,
                Self::FilterInput(input) => Self::update_filter(input, add_assets),
                Self::InputAmount(asset_address, symbol, amount) => {
                    Self::change_asset_amount(asset_address, symbol, amount, add_assets)
                }
                Self::InputMaxSelected => Self::set_max_amount_for_selected(add_assets, appdata),
                Self::SelectAllTokens => Self::select_all_tokens(add_assets, appdata),
                Self::SelectAsset(asset_address, symbol) => {
                    Self::add_selected_asset(asset_address, symbol, add_assets)
                }
                Self::UnselectAllTokens => {
                    add_assets.selected.clear();
                    add_assets.select_all = false
                }
                Self::UnselectAsset(asset_address) => {
                    Self::remove_selected_asset(asset_address, add_assets)
                }
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
        let fungibles = appdata
            .fungibles
            .get(&add_assets.from_account)
            .and_then(|fungibles| {
                Some(
                    fungibles
                        .iter()
                        .map(|fungible_asset| (&fungible_asset.resource_address, fungible_asset))
                        .collect::<HashMap<&ResourceAddress, &FungibleAsset>>(),
                )
            })
            .unwrap_or(HashMap::new());

        for (resource_address, (_symbol, amount)) in &mut add_assets.selected {
            let fungible_amount = fungibles
                .get(resource_address)
                .and_then(|fungible| Some(fungible.amount.clone()))
                .unwrap_or("0".to_string());

            *amount = fungible_amount;
        }
    }

    fn select_all_tokens(add_assets: &mut AddAssets, appdata: &mut AppData) {
        let fungibles = appdata.fungibles.get(&add_assets.from_account);

        if let Some(fungibles) = fungibles {
            for fungible in fungibles {
                if !add_assets.selected.contains_key(&fungible.resource_address) {
                    if let Some(resource) = appdata.resources.get(&fungible.resource_address) {
                        add_assets.selected.insert(
                            resource.address.clone(),
                            (resource.symbol.clone(), String::new()),
                        );
                    }
                }
            }
        }

        add_assets.select_all = true
    }

    fn change_asset_amount(
        asset: ResourceAddress,
        symbol: String,
        new_amount: String,
        add_assets: &mut AddAssets,
    ) {
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

    fn add_selected_asset(
        asset_address: ResourceAddress,
        symbol: String,
        add_assets: &mut AddAssets,
    ) {
        add_assets
            .selected
            .insert(asset_address, (symbol, "".to_string()));
    }

    fn remove_selected_asset(asset_address: ResourceAddress, add_assets: &mut AddAssets) {
        add_assets.select_all = false;
        add_assets.selected.remove_entry(&asset_address);
    }

    fn submit_selected_assets(add_assets: &mut AddAssets, recipients: &mut Vec<Recipient>) {
        for (resource_address, (symbol, amount)) in add_assets.selected.drain() {
            if let Some((_, old_amount)) = recipients[add_assets.recipient_index]
                .resources
                .get_mut(&resource_address)
            {
                *old_amount = amount
            } else {
                recipients[add_assets.recipient_index]
                    .resources
                    .insert(resource_address, (symbol, amount));
            }
        }
    }
}
