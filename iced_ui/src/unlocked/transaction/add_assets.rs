use std::{collections::HashMap, str::FromStr};

use iced::{
    widget::{self, button, checkbox, column, container, image::Handle, row, text, Container, TextInput},
    Element, Length, Padding, Task,
};

use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use types::{
    address::{AccountAddress, Address, ResourceAddress},
    assets::FungibleAsset,
};
use wallet::{Unlocked, Wallet};

use crate::{app::AppData, app::AppMessage, unlocked::app_view};

use super::transaction_view::{self, Recipient};

#[derive(Debug, Clone)]
pub enum Message {
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

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::TransactionMessage(
            super::transaction_view::Message::AddAssetsMessage(self),
        ))
    }
}

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
    pub selected: HashMap<ResourceAddress, (String, String)>,
    pub select_all: bool,
}

impl<'a> AddAssets {
    pub fn new(
        from_account: AccountAddress,
        recipient_index: usize,
        selected: HashMap<ResourceAddress, (String, String)>,
    ) -> Self {
        Self {
            tab: AssetTab::Tokens,
            from_account,
            recipient_index,
            filter: String::new(),
            selected,
            select_all: false,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        recipients: &'a mut Vec<Recipient>,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        let mut command = Task::none();

        match message {
            Message::SetTab(tab) => self.tab = tab,
            Message::FilterInput(input) => self.update_filter(input),
            Message::InputAmount(asset_address, symbol, amount) => {
                self.change_asset_amount(asset_address, symbol, amount)
            }
            Message::InputMaxSelected => self.set_max_amount_for_selected(wallet),
            Message::SelectAllTokens => self.select_all_tokens(wallet),
            Message::SelectAsset(asset_address, symbol) => {
                self.add_selected_asset(asset_address, symbol)
            }
            Message::UnselectAllTokens => {
                self.selected.clear();
                self.select_all = false
            }
            Message::UnselectAsset(asset_address) => self.remove_selected_asset(asset_address),
            Message::SubmitAssets => {
                command = self.submit_selected_assets(recipients);
            }
        }

        command
    }

    fn update_filter(&mut self, mut input: String) {
        input.make_ascii_lowercase();
        self.filter = input
    }

    fn set_max_amount_for_selected(&mut self, wallet: &mut Wallet<Unlocked>) {
        let fungibles = wallet
            .fungibles()
            .get(&self.from_account)
            .and_then(|fungibles| {
                Some(
                    fungibles
                        .iter()
                        .map(|fungible_asset| (&fungible_asset.resource_address, fungible_asset))
                        .collect::<HashMap<&ResourceAddress, &FungibleAsset>>(),
                )
            })
            .unwrap_or(HashMap::new());

        for (resource_address, (_symbol, amount)) in &mut self.selected {
            let fungible_amount = fungibles
                .get(resource_address)
                .and_then(|fungible| Some(fungible.amount.clone()))
                .unwrap_or("0".to_string());

            *amount = fungible_amount;
        }
    }

    fn select_all_tokens(&mut self, wallet: &mut Wallet<Unlocked>) {
        let fungibles = wallet.fungibles().get(&self.from_account);

        if let Some(fungibles) = fungibles {
            for fungible in fungibles {
                if !self.selected.contains_key(&fungible.resource_address) {
                    if let Some(resource) = wallet.resources().get(&fungible.resource_address) {
                        self.selected.insert(
                            resource.address.clone(),
                            (resource.symbol.clone(), String::new()),
                        );
                    }
                }
            }
        }

        self.select_all = true
    }

    fn change_asset_amount(&mut self, asset: ResourceAddress, symbol: String, new_amount: String) {
        if new_amount.parse::<f32>().is_ok() {
            if let Some((_, amount)) = self.selected.get_mut(&asset) {
                *amount = new_amount
            } else {
                self.selected.insert(asset, (symbol, new_amount));
            }
        } else if new_amount.is_empty() {
            self.selected.remove(&asset);
        }
    }

    fn add_selected_asset(&mut self, asset_address: ResourceAddress, symbol: String) {
        self.selected
            .insert(asset_address, (symbol, "".to_string()));
    }

    fn remove_selected_asset(&mut self, asset_address: ResourceAddress) {
        self.select_all = false;
        self.selected.remove_entry(&asset_address);
    }

    fn submit_selected_assets(&mut self, recipients: &mut Vec<Recipient>) -> Task<AppMessage> {
        for (resource_address, (symbol, amount)) in self.selected.drain() {
            if let Some((_, old_amount)) = recipients[self.recipient_index]
                .resources
                .get_mut(&resource_address)
            {
                *old_amount = amount
            } else {
                recipients[self.recipient_index]
                    .resources
                    .insert(resource_address, (symbol, amount));
            }
        }

        Task::perform(async {}, |_| transaction_view::Message::OverView.into())
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        let header = text("Add Assets")
            .width(Length::Fill)
            .line_height(2.)
            .size(16)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center);

        let space = widget::Space::new(1, 20);

        let search_field = TextInput::new("Search token", &self.filter)
            .line_height(1.5)
            .size(12)
            .width(250)
            .on_input(|input| Message::FilterInput(input).into());
        let search_field = container(search_field)
            .center_x(Length::Fill)
            .height(Length::Shrink);

        let space2 = widget::Space::new(1, 10);

        let tokens_button = button(
            text("Tokens")
                .size(12)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(80)
        .height(30)
        .on_press(Message::SetTab(AssetTab::Tokens).into());

        let nfts_button = button(
            text("NFTs")
                .size(12)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(80)
        .height(30)
        .on_press(Message::SetTab(AssetTab::NFTs).into());

        let buttons = row![tokens_button, nfts_button]
            .spacing(100)
            .align_y(iced::Alignment::Center);
        let buttons = container(buttons).center_x(Length::Fill);

        let mut amounts_within_limits = true;

        let asset_tab = match self.tab {
            AssetTab::Tokens => self.tokens_tab(wallet, &mut amounts_within_limits),
            AssetTab::NFTs => self.nfts_tab(wallet, &mut amounts_within_limits),
        };

        let submit_button = button(
            text("Submit")
                .size(16)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .width(150)
        .height(40)
        .on_press_maybe(if self.selected.is_empty() | !amounts_within_limits {
            None
        } else {
            Some(Message::SubmitAssets.into())
        });

        let bottom_button_container = container(submit_button)
            .center_x(Length::Fill)
            .center_y(Length::Shrink);

        column![
            header,
            space,
            search_field,
            space2,
            buttons,
            asset_tab,
            bottom_button_container
        ]
        .spacing(5)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn tokens_tab(
        &'a self,
        wallet: &'a Wallet<Unlocked>,
        within_limits: &mut bool,
    ) -> Container<'a, AppMessage> {
        let headers: Element<'a, AppMessage> = {
            let token_name = text("Token").size(12);

            let space = widget::Space::new(Length::Fill, 1);

            let balance = text("Available balance").size(12);

            let amount = container(
                button(text("Set max").size(12))
                    .padding(0)
                    .style(button::text)
                    .on_press(Message::InputMaxSelected.into()),
            )
            .center_x(85)
            .center_y(Length::Fill);

            let selected = checkbox("", self.select_all).size(12).on_toggle(|select| {
                if select {
                    Message::SelectAllTokens.into()
                } else {
                    Message::UnselectAllTokens.into()
                }
            });

            let header_row = row![token_name, space, balance, amount, selected]
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(10)
                .padding(5)
                .align_y(iced::Alignment::Center);

            container(header_row)
                .width(Length::Fill)
                .height(30)
                .padding(Padding {
                    right: 15.,
                    ..Padding::ZERO
                })
                .into()
        };

        let fungibles = wallet.fungibles().get(&self.from_account);

        let elements: Vec<Element<'a, AppMessage>> = match fungibles {
            Some(fungibles) => {
                fungibles
                    .into_iter()
                    .filter_map(|token| {
                        if let Some(resource) = wallet.resources().get(&token.resource_address) {
                            if resource.name.to_ascii_lowercase().contains(&self.filter)
                                || resource.symbol.to_ascii_lowercase().contains(&self.filter)
                                || resource.address.as_str().contains(&self.filter)
                            {
                                Some((token, resource))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .map(|(token, resource)| {
                        let selected = self
                            .selected
                            .get(&token.resource_address)
                            .and_then(|selected| Some((true, selected.1.as_str())))
                            .unwrap_or((self.select_all, ""));

                        let icon: Element<'a, AppMessage> = wallet
                            .resource_icons()
                            .get(&token.resource_address)
                            .and_then(|bytes| {
                                Some(widget::image(Handle::from_bytes(bytes.clone())).width(40).height(40).into())
                            })
                            .unwrap_or(
                                container(text(Bootstrap::Image).font(BOOTSTRAP_FONT).size(30))
                                    .center_x(40)
                                    .center_y(40)
                                    .into(),
                            );

                        let name = text(&resource.name).size(12);
                        let symbol = text(&resource.symbol).size(10);
                        let name_and_symbol = column![name, symbol].spacing(2);

                        let space = widget::Space::new(Length::Fill, 1);

                        let balance =
                            button(text(format!("{} {}", &token.amount, resource.symbol)).size(12))
                                .style(button::text)
                                .on_press(
                                    Message::InputAmount(
                                        resource.address.clone(),
                                        resource.symbol.clone(),
                                        token.amount.clone(),
                                    )
                                    .into(),
                                );

                        // let token_address = resource.address.clone();
                        // let token_symbol = resource.symbol.clone();
                        let amount = TextInput::new("Amount", selected.1)
                            .size(10)
                            .width(80)
                            .style(styles::text_input::asset_amount)
                            .on_input(move |input| {
                                Message::InputAmount(
                                    token.resource_address.clone(),
                                    resource.symbol.clone(),
                                    input,
                                )
                                .into()
                            });

                        let checkbox = checkbox("", selected.0).size(12).on_toggle(move |select| {
                            if select {
                                Message::SelectAsset(
                                    resource.address.clone(),
                                    resource.symbol.clone(),
                                )
                                .into()
                            } else {
                                Message::UnselectAsset(resource.address.clone()).into()
                            }
                        });

                        let asset = row![icon, name_and_symbol, space, balance, amount, checkbox]
                            .spacing(10)
                            .align_y(iced::Alignment::Center)
                            .width(Length::Fill)
                            .padding(5);

                        let rule = widget::Rule::horizontal(1);

                        let mut column = column![asset].width(Length::Fill);

                        if selected.0 {
                            if let Ok(decimal) = types::RadixDecimal::from_str(selected.1) {
                                if let Ok(token_amount) =
                                    types::RadixDecimal::from_str(token.amount.as_str())
                                {
                                    if decimal > token_amount {
                                        *within_limits = false;
                                        let warning = text("Amount exceeds available balance")
                                            .size(10)
                                            .line_height(1.5)
                                            .align_x(iced::alignment::Horizontal::Center);

                                        let container = container(warning)
                                            .width(Length::Fill)
                                            .padding(5)
                                            .align_x(iced::alignment::Horizontal::Right);

                                        column = column.push(container);
                                    }
                                }
                            }
                        }

                        column.push(rule).into()
                    })
                    .collect()
            }
            None => {
                // Create element for no assets found
                vec![]
            }
        };

        let scrollable = widget::scrollable(column(elements).padding(Padding {
            right: 15.,
            ..Padding::ZERO
        }))
        .style(styles::scrollable::vertical_scrollable)
        .height(Length::Shrink);

        container(column![headers, scrollable])
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
    }

    fn nfts_tab(
        &self,
        wallet: &'a Wallet<Unlocked>,
        within_limits: &mut bool,
    ) -> Container<'a, AppMessage> {
        let headers: Element<'a, AppMessage> = {
            let token_name = text("Token").size(12);

            let space = widget::Space::new(Length::Fill, 1);

            let balance = text("Available balance").size(12);

            let amount = container(
                button(text("Set max").size(12))
                    .padding(0)
                    .style(button::text)
                    .on_press(Message::InputMaxSelected.into()),
            )
            .center_x(85)
            .center_y(Length::Fill);

            let selected = checkbox("", self.select_all).size(12).on_toggle(|select| {
                if select {
                    Message::SelectAllTokens.into()
                } else {
                    Message::UnselectAllTokens.into()
                }
            });

            let header_row = row![token_name, space, balance, amount, selected]
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(10)
                .padding(5)
                .align_y(iced::Alignment::Center);

            container(header_row)
                .width(Length::Fill)
                .height(30)
                .padding(Padding {
                    right: 15.,
                    ..Padding::ZERO
                })
                .into()
        };

        let scrollable = widget::scrollable(column!().padding(Padding {
            right: 15.,
            ..Padding::ZERO
        }))
        .style(styles::scrollable::vertical_scrollable)
        .height(Length::Shrink);

        container(column![headers, scrollable])
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
    }
}
