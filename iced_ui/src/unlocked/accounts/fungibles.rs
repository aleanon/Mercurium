use std::{collections::btree_set, iter};

use deps::*;

use debug_print::debug_println;
use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, column, container, image::Handle, row, text, Button},
    Element, Length, Padding, Task,
};
use store::{DbError, IconsDb};
use wallet::{Unlocked, Wallet};

use crate::{app::AppMessage, styles, unlocked::app_view};
use types::{address::AccountAddress, assets::FungibleAsset};

use super::{
    account_view, accounts_view,
    fungible::{FungibleView, Icon},
};

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    SelectFungible(FungibleAsset),
    InsertFungibleImage(Vec<u8>),
    ImageNotFound,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::AccountsViewMessage(
            accounts_view::Message::AccountViewMessage(account_view::Message::FungiblesMessage(
                self,
            )),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Fungibles {
    pub account_addr: AccountAddress,
    pub selected: Option<FungibleView>,
}

impl<'a> Fungibles {
    pub fn new(account_addr: AccountAddress) -> Self {
        Self {
            account_addr,
            selected: None,
        }
    }
}

impl<'a> Fungibles {
    pub fn update(
        &mut self,
        message: Message,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        match message {
            Message::Back => self.back(wallet),
            Message::SelectFungible(fungible) => return self.select_fungible(fungible, wallet),
            Message::InsertFungibleImage(image_data) => self.insert_fungible_image(image_data),
            Message::ImageNotFound => {
                if let Some(fungible) = &mut self.selected {
                    fungible.image = Icon::None
                }
            }
        }
        Task::none()
    }

    fn back(&mut self, wallet: &'a mut Wallet<Unlocked>) {}

    fn select_fungible(
        &mut self,
        fungible: FungibleAsset,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        let address = fungible.resource_address.clone();
        self.selected = Some(FungibleView::new(fungible, Icon::Loading));

        let network = wallet.settings().network;
        Task::perform(
            async move {
                let icon_cache = IconsDb::get(network).ok_or(DbError::DatabaseNotLoaded)?;
                icon_cache.get_resource_icon(address).await
            },
            |result| match result {
                Ok((_, icon_data)) => Message::InsertFungibleImage(icon_data).into(),
                Err(_) => {
                    debug_println!("Could not find image");
                    Message::ImageNotFound.into()
                }
            },
        )
    }

    fn insert_fungible_image(&mut self, image_data: Vec<u8>) {
        if let Some(ref mut fungible_view) = self.selected {
            fungible_view.image = Icon::Some(Handle::from_bytes(image_data))
        }
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> iced::Element<'a, AppMessage> {
        match &self.selected {
            Some(fungible_view) => fungible_view.view(wallet),
            None => {
                let elements = wallet
                    .wallet_data()
                    .resource_data
                    .fungibles
                    .get(&self.account_addr)
                    .into_iter()
                    .flatten()
                    .map(|fungible| {
                        let button = Self::fungible_list_button(fungible, wallet)
                            .on_press(Message::SelectFungible(fungible.clone()).into());

                        let button_container =
                            container(button).style(styles::container::asset_list_item);

                        let rule = widget::Rule::horizontal(2);

                        column![button_container, rule].into()
                    });

                let column = column(elements)
                    .align_x(iced::Alignment::Center)
                    .padding(Padding {
                        right: 15.,
                        ..Padding::ZERO
                    })
                    .width(Length::Fill);

                widget::scrollable(column)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .style(styles::scrollable::vertical_scrollable_secondary)
                    .into()
            }
        }
    }

    fn fungible_list_button(
        fungible: &FungibleAsset,
        wallet: &'a Wallet<Unlocked>,
    ) -> Button<'a, AppMessage> {
        let icon: iced::Element<'a, AppMessage> =
            match wallet.resource_icons().get(&fungible.resource_address) {
                Some(bytes) => widget::image(Handle::from_bytes(bytes.clone()))
                    .width(40)
                    .height(40)
                    .into(),
                None => container(text(Bootstrap::Image).font(BOOTSTRAP_FONT).size(30))
                    .center_x(40)
                    .center_y(40)
                    .into(),
            };
        let (name, symbol) = match wallet.resources().get(&fungible.resource_address) {
            Some(resource) => {
                let symbol;
                if resource.symbol.is_empty() {
                    symbol = None;
                } else {
                    symbol = Some(resource.symbol.as_str());
                }
                (resource.name.as_str(), symbol)
            }
            None => ("NoName", None),
        };

        let mut name_and_symbol = column![text(name).size(16)]
            .spacing(3)
            .align_x(iced::Alignment::Start);
        name_and_symbol = name_and_symbol.push_maybe(symbol.and_then(|s| Some(text(s).size(14))));

        let list_button_content = row![
            icon,
            name_and_symbol,
            widget::Space::new(Length::Fill, 1),
            text(format!("{} {}", &fungible.amount, symbol.unwrap_or(""))).size(18)
        ]
        .padding(Padding {
            left: 10.,
            right: 10.,
            bottom: 5.,
            top: 5.,
        })
        .spacing(15)
        .align_y(iced::Alignment::Center);

        widget::button(list_button_content).style(styles::button::asset_list_button)
    }
}
