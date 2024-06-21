use std::collections::{BTreeMap, BTreeSet};

use crate::app::App;
use crate::message::app_view_message::accounts_message::account_message::AccountViewMessage;
use crate::message::app_view_message::AppViewMessage;
use crate::message::common_message::CommonMessage;
use crate::message::Message;
use crate::view::app_view::overlay::receive::Receive;
use crate::view::app_view::overlay::Overlay;
use iced::theme;
use iced::{
    alignment,
    widget::{
        self, column, container, row,
        scrollable::{self, Properties},
        text, Button,
    },
    Element, Length, Padding,
};

use ravault_iced_theme::styles;
use ravault_iced_theme::styles::container::AssetListItem;

use store::Db;
use types::assets::FungibleAsset;
use types::{Account, AccountAddress, EntityAccount, NonFungibles};

use super::fungibles_view::FungiblesView;

#[derive(Debug, Clone)]
pub enum AssetView {
    Fungibles(FungiblesView),
    NonFungibles,
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
        fungible_assets: BTreeSet<FungibleAsset>,
    ) -> Self {
        Self {
            name,
            address: address.clone(),
            // fungibles: None,
            // non_fungibles: None,
            view: AssetView::Fungibles(FungiblesView::new(address)),
        }
    }

    pub fn from_account(account: &Account) -> Self {
        Self {
            name: account.name.clone(),
            address: account.address.clone(),
            // fungibles: account.fungibles,
            // non_fungibles: account.non_fungibles,
            view: AssetView::Fungibles(FungiblesView::new(account.address.clone())),
        }
    }
}

impl<'a> AccountView {
    pub fn view(&self, app: &'a App) -> Element<'a, Message> {
        let mut accounts = app.app_data.db.get_accounts().unwrap_or(BTreeMap::new());

        let account = accounts.remove(&self.address).unwrap_or(Account::none());

        let account_name = text(&self.name)
            .size(20)
            .vertical_alignment(alignment::Vertical::Bottom);

        let account_address = text(self.address.truncate())
            .size(15)
            .vertical_alignment(iced::alignment::Vertical::Bottom);

        let icon = text(iced_aw::Bootstrap::Copy)
            .font(iced_aw::BOOTSTRAP_FONT)
            .size(15);

        let account_address_button = Button::new(row!(account_address, icon).spacing(5))
            .style(iced::theme::Button::Text)
            .on_press(Message::Common(CommonMessage::CopyToClipBoard(
                self.address.to_string(),
            )));

        let name_address_row = widget::row![
            account_name,
            widget::Space::new(Length::Fill, 1),
            account_address_button
        ]
        .align_items(iced::Alignment::End);

        let name_address = container(name_address_row).width(Length::FillPortion(6));
        let top_row = widget::row![
            widget::Space::new(Length::FillPortion(2), 1),
            name_address,
            widget::Space::new(Length::FillPortion(2), 1)
        ];

        let history_button = Self::nav_button("History");

        let transfer_button =
            Self::nav_button("Send").on_press(AppViewMessage::NewTransaction(Some(account)).into());

        let receive_button = Self::nav_button("Receive").on_press(
            AppViewMessage::SpawnOverlay(Overlay::Receive(Receive::new(self.address.clone())))
                .into(),
        );
        //TODO: On press spawn modal with qr code with accound address and the address written out with a copy button

        let nav_button_row = row![history_button, transfer_button, receive_button]
            // .width(Length::Shrink)
            // .height(Length::Shrink)
            .spacing(20);

        let nav_button_cont = container(nav_button_row).center_x().width(Length::Fill);
        // .height(Length::Shrink);

        let mut fung_button = Self::select_asset_button("Tokens")
            .on_press(AccountViewMessage::FungiblesView(self.address.clone()).into());

        let mut nft_button = Self::select_asset_button("NFTs")
            .on_press(AccountViewMessage::NonFungiblesView(self.address.clone()).into());

        let assets = match self.view {
            AssetView::Fungibles(ref fungibles_view) => {
                fung_button =
                    fung_button.style(theme::Button::custom(styles::button::GeneralSelectedButton));
                fungibles_view.view(app)
            }
            AssetView::NonFungibles => {
                nft_button =
                    nft_button.style(theme::Button::custom(styles::button::GeneralSelectedButton));
                self.view_non_fungibles(&app)
            }
        };

        let asset_button_row = row![fung_button, nft_button].spacing(100);
        let asset_button_cont = container(asset_button_row).center_x().width(Length::Fill);

        let col = column![top_row, nav_button_cont, asset_button_cont, assets]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(10)
            .align_items(iced::Alignment::Center);

        container(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    // fn account_header() -> Container<'a, Message> {}

    pub fn view_non_fungibles(&self, app: &'a App) -> iced::Element<'a, Message> {
        let non_fungibles = app.app_data.non_fungibles.get(&self.address);

        let column = {
            //Each non-fungible is turned into an element

            let mut elements: Vec<Element<'a, Message>> = Vec::new();

            if let Some(non_fungibles) = non_fungibles {
                for non_fungible in non_fungibles {
                    let icon: iced::Element<'a, Message> = match app
                        .app_data
                        .resource_icons
                        .get(&non_fungible.resource_address)
                    {
                        Some(handle) => widget::image(handle.clone()).width(40).height(40).into(),
                        None => widget::Space::new(40, 40).into(),
                    };

                    let name = app
                        .app_data
                        .resources
                        .get(&non_fungible.resource_address)
                        .and_then(|no_fungible| Some(no_fungible.name.as_str()))
                        .unwrap_or("NoName");

                    let symbol = text(name)
                        .size(12)
                        .height(15)
                        .vertical_alignment(iced::alignment::Vertical::Center)
                        .horizontal_alignment(iced::alignment::Horizontal::Left)
                        .width(Length::Fill);

                    let nr_of_nfts = text(non_fungible.nfids.nr_of_nfts())
                        .size(10)
                        .height(15)
                        .vertical_alignment(iced::alignment::Vertical::Center)
                        .horizontal_alignment(iced::alignment::Horizontal::Right)
                        .width(Length::Shrink);

                    let col = column![symbol, nr_of_nfts]
                        .align_items(iced::Alignment::Center)
                        .width(Length::Fill)
                        .height(Length::Shrink);

                    let row = row![icon, col]
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .padding(5)
                        .spacing(5)
                        .align_items(iced::Alignment::Center);

                    let button = widget::button(row)
                        .width(Length::Fill)
                        .height(50)
                        .on_press(Message::None)
                        .style(theme::Button::Text);

                    let container = container(button).style(AssetListItem::style);

                    elements.push(container.into())
                }
            } else {
                elements.push(text("No non_fungibles found").into())
            }

            column(elements)
                .align_items(iced::Alignment::Center)
                .padding(Padding {
                    right: 15.,
                    ..Padding::ZERO
                })
                .width(Length::Fill)
        };

        widget::scrollable(column)
            .direction(scrollable::Direction::Vertical(Properties::default()))
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn nav_button(name: &str) -> Button<Message> {
        Self::button(name).height(35).width(160)
    }

    fn select_asset_button(name: &str) -> Button<Message> {
        Self::button(name).height(30).width(120)
    }

    fn button(name: &str) -> Button<Message> {
        widget::Button::new(
            text(name)
                .size(15)
                .vertical_alignment(iced::alignment::Vertical::Center)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .height(Length::Fill)
                .width(Length::Fill),
        )
        .style(theme::Button::custom(styles::button::GeneralButton))
    }
}
