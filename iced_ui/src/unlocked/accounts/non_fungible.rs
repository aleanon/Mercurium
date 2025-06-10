use std::collections::HashMap;

use deps::{
    debug_print::debug_println,
    iced::{
        alignment::Horizontal,
        widget::{button, column, scrollable, text::Wrapping, Rule, Space},
        Task,
    },
    *,
};

use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, container, image::Handle, row, text, Container},
    Element, Length, Padding,
};
use store::{DbError, IconsDb};
use wallet::{Unlocked, Wallet};

use crate::{
    app::AppMessage,
    common,
    unlocked::accounts::{self, account_view, non_fungibles},
};
use types::{
    address::Address,
    assets::{NonFungibleAsset, NFT},
    Resource,
};

const FUNGIBLE_VIEW_WIDTH: Length = Length::Fixed(300.);

#[derive(Debug, Clone)]
pub enum Message {
    ResourceIcon(Icon),
    ImageLoaded(String, Icon),
    FailedToGetImage(String),
    SelectNFT(String),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(crate::unlocked::app_view::Message::AccountsViewMessage(
            accounts::accounts_view::Message::AccountViewMessage(
                account_view::Message::NonFungiblesMessage(
                    non_fungibles::Message::NonFungibleMessage(self),
                ),
            ),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Icon {
    None,
    Loading,
    Some(Handle),
}

#[derive(Debug, Clone)]
pub struct NonFungible {
    pub non_fungible_asset: NonFungibleAsset,
    pub nft_images: HashMap<String, (Icon, String)>,
    pub image: Icon,
    pub selected_nft: Option<String>,
}

impl<'a> NonFungible {
    pub fn new(
        non_fungible: NonFungibleAsset,
        wallet: &mut Wallet<Unlocked>,
    ) -> (Self, Task<AppMessage>) {
        let nfid_images: HashMap<String, (Icon, String)> = non_fungible
            .nfids
            .iter()
            .map(|nfid| {
                let (icon, url) = nfid
                    .nfdata
                    .iter()
                    .find_map(|nfdata| {
                        if nfdata.key == "key_image_url" {
                            Some((Icon::Loading, nfdata.value.clone()))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((Icon::None, String::new()));
                (nfid.id.clone(), (icon, url))
            })
            .collect();

        let mut load_images = nfid_images
            .iter()
            .filter(|(_, (icon, _))| icon == &Icon::Loading)
            .map(|(nfid, (_, url))| {
                let nfid_clone = nfid.clone();
                let url_clone = url.clone();
                Task::perform(
                    async move {
                        let image_handle =
                            handles::image::download::download_and_resize_icon(&url_clone)
                                .await
                                .and_then(|image| Some(Handle::from_bytes(image)));
                        match image_handle {
                            Some(handle) => (nfid_clone, Icon::Some(handle)),
                            None => (nfid_clone, Icon::None),
                        }
                    },
                    |(nfid, icon)| match &icon {
                        Icon::Some(_) => Message::ImageLoaded(nfid, icon).into(),
                        _ => Message::FailedToGetImage(nfid).into(),
                    },
                )
            })
            .collect::<Vec<_>>();

        let network = wallet.settings().network;
        let address = non_fungible.resource_address.clone();
        let get_asset_icon = Task::perform(
            async move {
                let icon_cache = IconsDb::get(network).ok_or(DbError::DatabaseNotLoaded)?;
                icon_cache.get_resource_icon(address).await
            },
            |result| match result {
                Ok((_, icon_data)) => {
                    Message::ResourceIcon(Icon::Some(Handle::from_bytes(icon_data))).into()
                }
                Err(_) => {
                    debug_println!("Could not find image");
                    Message::ResourceIcon(Icon::None).into()
                }
            },
        );

        load_images.push(get_asset_icon);

        (
            Self {
                non_fungible_asset: non_fungible,
                nft_images: nfid_images,
                image: Icon::Loading,
                selected_nft: None,
            },
            Task::batch(load_images),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<AppMessage> {
        match message {
            Message::FailedToGetImage(nfid) => {
                if let Some((icon, _)) = self.nft_images.get_mut(&nfid) {
                    *icon = Icon::None;
                }
            }
            Message::ImageLoaded(nfid, new_icon) => {
                if let Some((icon, _)) = self.nft_images.get_mut(&nfid) {
                    *icon = new_icon;
                }
            }
            Message::ResourceIcon(icon) => self.image = icon,
            Message::SelectNFT(nfid) => self.selected_nft = Some(nfid),
        }
        Task::none()
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> iced::Element<'a, AppMessage> {
        let resource = wallet
            .wallet_data()
            .resource_data
            .resources
            .get(&self.non_fungible_asset.resource_address);

        let content = match &self.selected_nft {
            Some(nfid) => self.nft_details(nfid),
            None => self.non_fungible_asset(resource),
        };

        container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(Padding::ZERO.right(30).left(30))
            .max_width(1000)
            .into()
    }

    fn non_fungible_asset(&'a self, resource: Option<&'a Resource>) -> Container<'a, AppMessage> {
        let name = text(
            resource
                .and_then(|resource| Some(resource.name.as_str()))
                .unwrap_or("NoName"),
        )
        .size(15)
        .line_height(2.);

        let image = create_image(&self.image).center_x(150).center_y(150);

        let amount = row![
            text(&self.non_fungible_asset.nfids.len())
                .line_height(1.5)
                .size(12)
                .width(Length::Shrink),
            text(
                resource
                    .and_then(|resource| Some(resource.symbol.as_str()))
                    .unwrap_or("")
            )
            .line_height(1.5)
            .size(12)
            .width(Length::Shrink)
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center);

        let image_name_amount = widget::column![name, image, amount]
            .align_x(iced::Alignment::Center)
            .spacing(10)
            .padding(Padding {
                bottom: 5.,
                ..Padding::from(0)
            });

        let description = text(
            resource
                .and_then(|resource| Some(resource.description.as_str()))
                .unwrap_or("No description"),
        )
        .line_height(1.5)
        .size(12)
        .width(Length::Fill);

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let address = row![
            text("Address:").size(12),
            space,
            text(self.non_fungible_asset.resource_address.truncate()).size(12)
        ]
        .padding(Padding {
            top: 5.,
            ..Padding::from(0)
        });

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let current_supply = row![
            text("Current Supply:").size(12),
            space,
            text(
                resource
                    .and_then(|resource| Some(resource.current_supply.as_str()))
                    .unwrap_or("Unknown")
            )
            .size(12),
        ];

        let rule = widget::Rule::horizontal(2);
        let rule2 = widget::Rule::horizontal(2);

        let col = widget::column![
            image_name_amount,
            rule,
            description,
            rule2,
            address,
            current_supply,
        ]
        .spacing(15)
        .align_x(iced::Alignment::Center)
        .height(Length::Shrink)
        .padding(Padding::from([0, 10]));

        let mut nfids = iced::widget::Grid::new().spacing(10).fluid(250);

        for nfid in self.non_fungible_asset.nfids.iter() {
            let icon = self
                .nft_images
                .get(&nfid.id)
                .and_then(|(icon, _)| Some(icon))
                .unwrap_or(&Icon::None);

            let nfid_card = nft_card(nfid, icon).max_width(250).max_height(300);

            nfids = nfids.push(nfid_card);
        }

        let scrollable = widget::scrollable(
            column![col, nfids]
                .spacing(20)
                .align_x(Horizontal::Center)
                .padding(15),
        )
        .style(styles::scrollable::vertical_scrollable);

        container(scrollable)
            .padding(5)
            .style(styles::container::token_container)
    }

    fn nft_details(&'a self, nft_id: &'a String) -> Container<'a, AppMessage> {
        let Some(nft) = self
            .non_fungible_asset
            .nfids
            .iter()
            .find(|nft| &nft.id == nft_id)
        else {
            return container(text("NFT not found").size(20))
                .center_x(Length::Fill)
                .center_y(Length::Shrink);
        };

        let name = nft
            .nfdata
            .iter()
            .find_map(|nfdata| {
                if nfdata.key == "name" {
                    Some(nfdata.value.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("");

        let header = container(text(name).size(20))
            .padding(10)
            .center_x(Length::Fill);

        let icon = self
            .nft_images
            .get(nft_id)
            .and_then(|(i, _)| Some(i))
            .unwrap_or(&Icon::None);

        let image = create_image(icon).center_x(150).center_y(150);

        let description = nft
            .nfdata
            .iter()
            .find_map(|nft_data| {
                if nft_data.key == "description" {
                    Some(text(nft_data.value.as_str()).size(12))
                } else {
                    None
                }
            })
            .unwrap_or(text(""));

        let nft_name = row![
            text("Name").size(12),
            Space::new(Length::Fill, 1),
            text(name).size(13)
        ];

        let trim: &[_] = &['{', '}', '<', '>'];
        let nft_id = nft_id.trim_matches(trim);
        let nft_id: Element<'_, AppMessage> = match nft_id.len() {
            len @ 22.. => row![
                text(&nft_id[0..8]).size(13),
                text("...").size(13),
                text(&nft_id[len - 5..len]).size(13)
            ]
            .into(),
            _ => text(nft_id).size(13).into(),
        };

        let nft_id = row![
            text("ID").size(12),
            Space::new(Length::Fill, 1),
            button(row![nft_id, text(Bootstrap::Copy).font(BOOTSTRAP_FONT).size(13)].spacing(5))
                .style(button::text)
                .on_press(common::Message::CopyToClipBoard(nft.id.clone()).into())
        ];

        let mut metadata = column![].spacing(10);

        for nft_data in &nft.nfdata {
            if nft_data.key == "name"
                || nft_data.key == "description"
                || nft_data.key == "key_image_url"
            {
                continue;
            };

            let data = row![
                text(nft_data.key.as_str()).size(12),
                Space::new(Length::Fill, 1),
                text(nft_data.value.as_str()).size(13)
            ];
            metadata = metadata.push(data);
        }

        let content = column![
            header,
            image,
            Rule::horizontal(1),
            description,
            Rule::horizontal(1),
            nft_id,
            nft_name,
            Rule::horizontal(1),
            metadata
        ]
        .spacing(10)
        .align_x(Horizontal::Center)
        .padding(15);

        container(scrollable(content).style(styles::scrollable::vertical_scrollable))
            .center_x(Length::Fill)
            .style(styles::container::token_container)
            .max_width(800)
    }
}

fn create_image(icon: &Icon) -> Container<'_, AppMessage> {
    match icon {
        Icon::Some(handle) => container(widget::image(handle.clone()).expand(true))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        Icon::Loading => container("").width(Length::Fill).height(Length::Fill),
        Icon::None => container(text(Bootstrap::Image).font(BOOTSTRAP_FONT).size(100))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    }
}

fn nft_card<'a>(nft: &'a NFT, icon: &'a Icon) -> Container<'a, AppMessage> {
    let image = create_image(&icon);

    let nft_name = nft
        .nfdata
        .iter()
        .find_map(|nfdata| {
            if nfdata.key == "name" {
                Some(text(&nfdata.value).size(12))
            } else {
                None
            }
        })
        .unwrap_or(text(""));

    let trim: &[_] = &['{', '}', '<', '>'];
    let nft_id = nft.id.trim_matches(trim);
    let nft_id = match nft_id.len() {
        len @ 22.. => container(row![
            text(&nft_id[0..8]).size(12),
            text("...").size(12),
            text(&nft_id[len - 5..len]).size(12)
        ]),
        _ => container(text(nft_id)),
    };

    let content = column![image, nft_name, nft_id]
        .spacing(10)
        .align_x(Horizontal::Left);

    let content = button(content)
        .on_press(Message::SelectNFT(nft.id.clone()).into())
        .padding(10)
        .style(styles::button::nft_button);

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
}

fn nft_data_row<'a>(key: &'a str, value: &'a str) -> Element<'a, AppMessage> {
    row![
        text(key).size(10),
        widget::Space::new(Length::Fill, 1),
        text(value).size(10),
    ]
    .into()
}

fn nft() {}
