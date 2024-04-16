use std::{collections::HashMap, str::FromStr};

use iced::{
    theme,
    widget::{
        self, button, checkbox, column, container, row, text, Container, TextInput,
    },
    Element, Length, Padding,
};

use ravault_iced_theme::styles;
use types::{AccountAddress, Fungibles, NonFungibles, ResourceAddress};

use crate::{
    message::{
        app_view_message::transaction_message::add_assets_message::AddAssetsMessage, Message,
    },
    App,
};

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

    pub fn view(&'a self, app: &'a App) -> Element<'a, Message> {
        let header = text("Add Assets")
            .width(Length::Fill)
            .line_height(2.)
            .size(16)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .vertical_alignment(iced::alignment::Vertical::Center);

        let space = widget::Space::new(1, 20);

        let search_field = TextInput::new("Search token", &self.filter)
            .line_height(1.5)
            .size(12)
            .width(250)
            .on_input(|input| AddAssetsMessage::FilterInput(input).into());
        let search_field = container(search_field)
            .center_x()
            .width(Length::Fill)
            .height(Length::Shrink);

        let space2 = widget::Space::new(1, 10);

        let tokens_button = button(
            text("Tokens")
                .size(12)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
        .width(80)
        .height(30)
        .on_press(AddAssetsMessage::SetTab(AssetTab::Tokens).into());

        let nfts_button = button(
            text("NFTs")
                .size(12)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
        .width(80)
        .height(30)
        .on_press(AddAssetsMessage::SetTab(AssetTab::NFTs).into());

        let buttons = row![tokens_button, nfts_button]
            .spacing(100)
            .align_items(iced::Alignment::Center);
        let buttons = container(buttons).center_x().width(Length::Fill);

        let mut amounts_within_limits = true;

        let asset_tab = match self.tab {
            AssetTab::Tokens => self.tokens_tab(app, &mut amounts_within_limits),
            AssetTab::NFTs => self.nfts_tab(app, &mut amounts_within_limits),
        };

        let submit_button = button(
            text("Submit")
                .size(16)
                .width(Length::Fill)
                .height(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .vertical_alignment(iced::alignment::Vertical::Center),
        )
        .width(150)
        .height(40)
        .on_press_maybe(if self.selected.is_empty() | !amounts_within_limits {
            None
        } else {
            Some(AddAssetsMessage::SubmitAssets.into())
        });

        let bottom_button_container = container(submit_button)
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Shrink);

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

    fn tokens_tab(&'a self, app: &'a App, within_limits: &mut bool) -> Container<'a, Message> {
        let fungibles = app
            .app_data
            .db
            .get_fungibles_by_account(&self.from_account)
            .unwrap_or(Fungibles::new());

        let headers: Element<'a, Message> = {
            let token_name = text("Token").size(12);

            let space = widget::Space::new(Length::Fill, 1);

            let balance = text("Available balance").size(12);

            let amount = container(
                button(text("Set max").size(12))
                    .padding(0)
                    .style(theme::Button::Text)
                    .on_press(AddAssetsMessage::InputMaxSelected.into()),
            )
            .width(85)
            .height(Length::Fill)
            .center_x()
            .center_y();

            let selected = checkbox("", self.select_all).size(12).on_toggle(|select| {
                if select {
                    AddAssetsMessage::SelectAllTokens.into()
                } else {
                    AddAssetsMessage::UnselectAllTokens.into()
                }
            });

            let row = row![token_name, space, balance, amount, selected]
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(10)
                .padding(5)
                .align_items(iced::Alignment::Center);

            container(row)
                .width(Length::Fill)
                .height(30)
                .padding(Padding {
                    right: 15.,
                    ..Padding::ZERO
                })
                .into()
        };

        let elements: Vec<Element<'a, Message>> = fungibles
            .into_iter()
            .filter(|token| {
                token.name.to_ascii_lowercase().contains(&self.filter)
                    || token.symbol.to_ascii_lowercase().contains(&self.filter)
                    || token.address.as_str().contains(&self.filter)
            })
            .map(|token| {
                let selected = self
                    .selected
                    .get(&token.address)
                    .and_then(|selected| Some((true, selected.1.as_str())))
                    .unwrap_or((self.select_all, ""));

                let icon: Element<'a, Message> = app
                    .appview
                    .resource_icons
                    .get(&token.address)
                    .and_then(|handle| {
                        Some(widget::image(handle.clone()).width(40).height(40).into())
                    })
                    .unwrap_or(
                        container(
                            text(iced_aw::BootstrapIcon::Image)
                                .font(iced_aw::BOOTSTRAP_FONT)
                                .size(30),
                        )
                        .width(40)
                        .height(40)
                        .center_x()
                        .center_y()
                        .into(),
                    );

                let name = text(&token.name).size(12);
                let symbol = text(&token.symbol).size(10);
                let name_and_symbol = column![name, symbol].spacing(2);

                let space = widget::Space::new(Length::Fill, 1);

                let token_balance = token.amount.to_string();
                let balance = button(text(format!("{} {}", &token_balance, token.symbol)).size(12))
                    .style(theme::Button::Text)
                    .on_press(
                        AddAssetsMessage::InputAmount(
                            token.address.clone(),
                            token.symbol.clone(),
                            token_balance,
                        )
                        .into(),
                    );

                let token_address = token.address.clone();
                let token_symbol = token.symbol.clone();
                let amount = TextInput::new("Amount", selected.1)
                    .size(10)
                    .width(80)
                    .style(theme::TextInput::Custom(Box::new(
                        styles::text_input::AssetAmount,
                    )))
                    .on_input(move |input| {
                        AddAssetsMessage::InputAmount(
                            token_address.clone(),
                            token_symbol.clone(),
                            input,
                        )
                        .into()
                    });

                let checkbox = checkbox("", selected.0).size(12).on_toggle(move |select| {
                    if select {
                        AddAssetsMessage::SelectAsset(token.address.clone(), token.symbol.clone())
                            .into()
                    } else {
                        AddAssetsMessage::UnselectAsset(token.address.clone()).into()
                    }
                });

                let asset = row![icon, name_and_symbol, space, balance, amount, checkbox]
                    .spacing(10)
                    .align_items(iced::Alignment::Center)
                    .width(Length::Fill)
                    .padding(5);

                let rule = widget::Rule::horizontal(1);

                let mut column = column![asset].width(Length::Fill);

                if selected.0 {
                    if let Ok(decimal) = types::RadixDecimal::from_str(selected.1) {
                        if decimal > token.amount.0 {
                            *within_limits = false;
                            let warning = text("Amount exceeds available balance")
                                .size(10)
                                .line_height(1.5)
                                .horizontal_alignment(iced::alignment::Horizontal::Center);

                            let container = container(warning)
                                .width(Length::Fill)
                                .padding(5)
                                .align_x(iced::alignment::Horizontal::Right);

                            column = column.push(container);
                        }
                    }
                }

                column.push(rule).into()
            })
            .collect();

        let scrollable = widget::scrollable(column(elements).padding(Padding {
            right: 15.,
            ..Padding::ZERO
        }))
        .style(theme::Scrollable::custom(styles::scrollable::Scrollable))
        .height(Length::Shrink);

        container(column![headers, scrollable])
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
    }

    fn nfts_tab(&self, app: &'a App, _within_limits: &mut bool) -> Container<'a, Message> {
        let _non_fungibles = app
            .app_data
            .db
            .get_non_fungibles_by_account(&self.from_account)
            .unwrap_or(NonFungibles::new());
        let scrollable = widget::scrollable(column!());

        container(scrollable)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
    }
}
