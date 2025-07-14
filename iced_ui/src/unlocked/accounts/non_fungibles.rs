use deps::*;

use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, column, container, image::Handle, row, text, Button},
    Element, Length, Padding, Task,
};
use wallet::{Unlocked, Wallet};

use crate::{
    app::AppMessage,
    styles,
    unlocked::{accounts::non_fungible, app_view},
};
use types::{address::AccountAddress, assets::NonFungibleAsset};

use super::{
    account_view, accounts_view,
    non_fungible::{Icon, NonFungible},
};

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    SelectNonFungible(NonFungibleAsset),
    InsertResourceIcon(Vec<u8>),
    ResourceIconNotFound,
    NonFungibleMessage(non_fungible::Message),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::AccountsViewMessage(
            accounts_view::Message::AccountViewMessage(account_view::Message::NonFungiblesMessage(
                self,
            )),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct NonFungibles {
    pub account_addr: AccountAddress,
    pub selected: Option<NonFungible>,
}

impl<'a> NonFungibles {
    pub fn new(account_addr: AccountAddress) -> Self {
        Self {
            account_addr,
            selected: None,
        }
    }
}

impl<'a> NonFungibles {
    pub fn update(
        &mut self,
        message: Message,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        match message {
            Message::Back => self.back(wallet),
            Message::SelectNonFungible(fungible) => return self.select_fungible(fungible, wallet),
            Message::InsertResourceIcon(image_data) => self.insert_fungible_image(image_data),
            Message::ResourceIconNotFound => {
                if let Some(fungible) = &mut self.selected {
                    fungible.image = Icon::None
                }
            }
            Message::NonFungibleMessage(message) => {
                if let Some(non_fungible) = &mut self.selected {
                    return non_fungible.update(message);
                }
            }
        }
        Task::none()
    }

    fn back(&mut self, wallet: &'a mut Wallet<Unlocked>) {}

    fn select_fungible(
        &mut self,
        non_fungible: NonFungibleAsset,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        let (selected, task) = NonFungible::new(non_fungible, wallet);
        self.selected = Some(selected);
        task
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
                let mut elements: Vec<Element<'a, AppMessage>> = Vec::new();

                if let Some(non_fungibles) = wallet
                    .wallet_data()
                    .resource_data
                    .non_fungibles
                    .get(&self.account_addr)
                {
                    for non_fungible in non_fungibles {
                        let button = Self::non_fungible_list_button(non_fungible, wallet)
                            .on_press(Message::SelectNonFungible(non_fungible.clone()).into());

                        let button_container =
                            container(button).style(styles::container::asset_list_item);

                        let rule = widget::Rule::horizontal(2);

                        elements.push(column![button_container, rule].into())
                    }
                } else {
                    // Push no widget to "elements"
                }

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

    fn non_fungible_list_button(
        fungible: &NonFungibleAsset,
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
            text(format!(
                "{} {}",
                &fungible.nfids.len(),
                symbol.unwrap_or("")
            ))
            .size(18)
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
