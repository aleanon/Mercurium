use deps::*;

use std::collections::BTreeSet;

use crate::app::AppMessage;
use crate::common;
use crate::unlocked::app_view;
use crate::unlocked::overlays::overlay::SpawnOverlay;

use iced::widget::button;
use iced::widget::image::Handle;
use iced::Task;
use iced::{
    alignment,
    widget::{self, column, container, row, text, Button},
    Element, Length, Padding,
};

use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use types::address::{AccountAddress, Address};
use types::assets::FungibleAsset;
use types::Account;
use wallet::Unlocked;
use wallet::Wallet;

use super::non_fungibles::NonFungibles;
use super::{accounts_view, non_fungibles};
use super::{fungibles, fungibles::Fungibles};

#[derive(Debug, Clone)]
pub enum Message {
    FungiblesView(AccountAddress),
    NonFungiblesView(AccountAddress),
    SelectFungible,
    SelectNonFungible {
        account_id: usize,
        non_fungible_id: usize,
    },
    SelectPoolUnit {
        account_id: usize,
        poolunit_id: usize,
    },
    FungiblesMessage(fungibles::Message),
    NonFungiblesMessage(non_fungibles::Message),
    //Transaction(Account),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::AccountsViewMessage(
            accounts_view::Message::AccountViewMessage(self),
        ))
    }
}

#[derive(Debug, Clone)]
pub enum AssetView {
    Tokens(Fungibles),
    NonFungibles(NonFungibles),
}

#[derive(Debug, Clone)]
pub enum Asset {
    Fungible(usize),
    NonFungible(usize),
}

#[derive(Debug, Clone)]
pub struct AccountView {
    pub name: String,
    pub address: AccountAddress,
    // pub fungibles: Option<Fungibles>,
    // pub non_fungibles: Option<NonFungibles>,
    pub view: AssetView,
}

impl<'a> AccountView {
    pub fn new(
        name: String,
        address: AccountAddress,
        // fungible_assets: BTreeSet<FungibleAsset>,
    ) -> Self {
        Self {
            name,
            address: address.clone(),
            // fungibles: None,
            // non_fungibles: None,
            view: AssetView::Tokens(Fungibles::new(address)),
        }
    }

    pub fn from_account(account: &Account) -> Self {
        Self {
            name: account.name.clone(),
            address: account.address.clone(),
            // fungibles: account.fungibles,
            // non_fungibles: account.non_fungibles,
            view: AssetView::Tokens(Fungibles::new(account.address.clone())),
        }
    }
}

impl<'a> AccountView {
    pub fn update(
        &mut self,
        message: Message,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        let mut command = Task::none();
        match message {
            Message::FungiblesView(account_address) => {
                self.view = AssetView::Tokens(Fungibles::new(account_address))
            }
            Message::NonFungiblesView(account_address) => {
                self.view = AssetView::NonFungibles(NonFungibles::new(account_address))
            }
            Message::SelectFungible => self.select_fungible(wallet),
            Message::SelectNonFungible {
                account_id,
                non_fungible_id,
            } => self.select_non_fungible(account_id, non_fungible_id, wallet),
            Message::SelectPoolUnit {
                account_id,
                poolunit_id,
            } => self.select_poolunit(account_id, poolunit_id, wallet),
            Message::FungiblesMessage(fungibles_message) => {
                if let AssetView::Tokens(fungibles) = &mut self.view {
                    command = fungibles.update(fungibles_message, wallet)
                }
            }
            Message::NonFungiblesMessage(nfts_message) => {
                if let AssetView::NonFungibles(non_fungibles) = &mut self.view {
                    command = non_fungibles.update(nfts_message, wallet)
                }
            } // Self::Transaction(account) => Self::transaction_from_account(account, app),
        }
        command
    }

    fn select_fungible(&mut self, _wallet: &'a mut Wallet<Unlocked>) {}

    fn select_non_fungible(
        &mut self,
        _account_id: usize,
        _non_fungible_id: usize,
        _wallet: &'a mut Wallet<Unlocked>,
    ) {
    }

    fn select_poolunit(
        &mut self,
        _account_id: usize,
        _poolunit_id: usize,
        _wallet: &'a mut Wallet<Unlocked>,
    ) {
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        // let mut accounts = appdata.db.get_accounts_map().unwrap_or(BTreeMap::new());
        let Some(account) = wallet
            .wallet_data()
            .resource_data
            .accounts
            .get(&self.address)
        else {
            return column!().into();
        };
        // let account = accounts.remove(&self.address).unwrap_or(Account::none());

        let account_name = text(&self.name)
            .size(20)
            .align_y(alignment::Vertical::Bottom);

        let account_address = text(self.address.truncate())
            .size(15)
            .align_y(iced::alignment::Vertical::Bottom);

        let icon = text(Bootstrap::Copy).font(BOOTSTRAP_FONT).size(15);

        let account_address_button = Button::new(row!(account_address, icon).spacing(5))
            .style(button::text)
            .on_press(AppMessage::Common(common::Message::CopyToClipBoard(
                self.address.to_string(),
            )));

        let name_address_row = widget::row![
            account_name,
            widget::Space::new(Length::Fill, 1),
            account_address_button
        ]
        .align_y(iced::Alignment::End);

        let name_address = container(name_address_row).width(Length::FillPortion(6));
        let top_row = widget::row![
            widget::Space::new(Length::FillPortion(2), 1),
            name_address,
            widget::Space::new(Length::FillPortion(2), 1)
        ];

        let history_button = Self::nav_button("History");

        let transfer_button = Self::nav_button("Send")
            .on_press(app_view::Message::NewTransaction(Some(account.clone())).into());

        let receive_button = Self::nav_button("Receive").on_press(
            app_view::Message::SpawnOverlay(SpawnOverlay::Receive(self.address.clone())).into(),
        );
        //TODO: On press spawn modal with qr code with accound address and the address written out with a copy button

        let nav_button_row = row![history_button, transfer_button, receive_button]
            // .width(Length::Shrink)
            // .height(Length::Shrink)
            .spacing(20);

        let nav_button_cont = container(nav_button_row).center_x(Length::Fill);
        // .height(Length::Shrink);

        let mut fung_button = Self::select_asset_button("Tokens")
            .on_press(Message::FungiblesView(self.address.clone()).into());

        let mut nft_button = Self::select_asset_button("NFTs")
            .on_press(Message::NonFungiblesView(self.address.clone()).into());

        let assets = match &self.view {
            AssetView::Tokens(fungibles_view) => {
                fung_button = fung_button.style(styles::button::general_selected_button);

                fungibles_view.view(wallet)
            }
            AssetView::NonFungibles(non_fungibles) => {
                nft_button = nft_button.style(styles::button::general_selected_button);

                non_fungibles.view(wallet)
                // self.view_non_fungibles(wallet)
            }
        };

        let asset_button_row = row![fung_button, nft_button].spacing(100);
        let asset_button_cont = container(asset_button_row).center_x(Length::Fill);

        let col = column![top_row, nav_button_cont, asset_button_cont, assets]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(10)
            .align_x(iced::Alignment::Center);

        container(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    // fn account_header() -> Container<'a, Message> {}

    pub fn view_non_fungibles(
        &self,
        wallet: &'a Wallet<Unlocked>,
    ) -> iced::Element<'a, AppMessage> {
        let non_fungibles = wallet.non_fungibles().get(&self.address);

        let column = {
            //Each non-fungible is turned into an element

            let mut elements: Vec<Element<'a, AppMessage>> = Vec::new();

            if let Some(non_fungibles) = non_fungibles {
                for non_fungible in non_fungibles {
                    let icon: iced::Element<'a, AppMessage> =
                        match wallet.resource_icons().get(&non_fungible.resource_address) {
                            Some(bytes) => widget::image(Handle::from_bytes(bytes.clone()))
                                .width(40)
                                .height(40)
                                .into(),
                            None => widget::Space::new(40, 40).into(),
                        };

                    let name = wallet
                        .resources()
                        .get(&non_fungible.resource_address)
                        .and_then(|no_fungible| Some(no_fungible.name.as_str()))
                        .unwrap_or("NoName");

                    let symbol = text(name)
                        .size(12)
                        .height(15)
                        .align_x(iced::alignment::Horizontal::Left)
                        .align_y(iced::alignment::Vertical::Center)
                        .width(Length::Fill);

                    let nr_of_nfts = text(non_fungible.nfids.nr_of_nfts())
                        .size(10)
                        .height(15)
                        .align_x(iced::alignment::Horizontal::Right)
                        .align_y(iced::alignment::Vertical::Center)
                        .width(Length::Shrink);

                    let col = column![symbol, nr_of_nfts]
                        .align_x(iced::Alignment::Center)
                        .width(Length::Fill)
                        .height(Length::Shrink);

                    let row = row![icon, col]
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .padding(5)
                        .spacing(5)
                        .align_y(iced::Alignment::Center);

                    let button = widget::button(row)
                        .width(Length::Fill)
                        .height(50)
                        .on_press(AppMessage::None)
                        .style(button::text);

                    let container = container(button).style(styles::container::asset_list_item);

                    elements.push(container.into())
                }
            } else {
                elements.push(text("No non_fungibles found").into())
            }

            column(elements)
                .align_x(iced::Alignment::Center)
                .padding(Padding {
                    right: 15.,
                    ..Padding::ZERO
                })
                .width(Length::Fill)
        };

        widget::scrollable(column)
            // .direction(scrollable::Direction::Vertical(Properties::default()))
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn nav_button(name: &str) -> Button<AppMessage> {
        Self::button(name).height(35).width(160)
    }

    fn select_asset_button(name: &str) -> Button<AppMessage> {
        Self::button(name).height(30).width(120)
    }

    fn button(name: &str) -> Button<AppMessage> {
        widget::Button::new(
            text(name)
                .size(15)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .height(Length::Fill)
                .width(Length::Fill),
        )
        .style(styles::button::general_button)
    }
}
