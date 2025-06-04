use deps::{scrypto::prelude::ContextualTryInto, *};

use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, container, image::Handle, row, text, Container},
    Element, Length, Padding,
};
use wallet::{Unlocked, Wallet};

use crate::app::AppMessage;
use types::{
    address::Address,
    assets::{FungibleAsset, NonFungibleAsset},
};

const FUNGIBLE_VIEW_WIDTH: Length = Length::Fixed(300.);

#[derive(Debug, Clone)]
pub enum Icon {
    None,
    Loading,
    Some(Handle),
}

#[derive(Debug, Clone)]
pub struct NonFungible {
    pub non_fungible: NonFungibleAsset,
    pub image: Icon,
}

impl<'a> NonFungible {
    pub fn new(fungible: NonFungibleAsset, image: Icon) -> Self {
        Self {
            non_fungible: fungible,
            image,
        }
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> iced::Element<'a, AppMessage> {
        let resource = wallet
            .wallet_data()
            .resource_data
            .resources
            .get(&self.non_fungible.resource_address);

        let name = text(
            resource
                .and_then(|resource| Some(resource.name.as_str()))
                .unwrap_or("NoName"),
        )
        .size(15)
        .line_height(2.);

        let image: Element<'a, AppMessage> = match &self.image {
            Icon::Some(handle) => widget::image(handle.clone()).into(),
            Icon::Loading => Container::new("").width(150).height(150).into(),
            Icon::None => container(text(Bootstrap::Image).font(BOOTSTRAP_FONT).size(100))
                .center_x(150)
                .center_y(150)
                .into(),
        };

        let amount = row![
            text(&self.non_fungible.nfids.len())
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
            text(self.non_fungible.resource_address.truncate()).size(12)
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

        let content = container(col)
            .padding(15)
            .style(styles::container::token_container);

        let scrollable = widget::scrollable(content).style(styles::scrollable::vertical_scrollable);

        let space_left = widget::Space::new(Length::Fill, Length::Fill);
        let space_right = widget::Space::new(Length::Fill, Length::Fill);

        container(scrollable)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(Padding::ZERO.right(30).left(30))
            .max_width(1000)
            .into()
    }
}
