use deps::{
    iced::widget::{Rule, horizontal_space},
    *,
};

use font_and_icons::{BOOTSTRAP_FONT, Bootstrap};
use iced::{
    Element, Length, Padding,
    widget::{self, Container, column, container, image::Handle, row, text},
};
use wallet::{Unlocked, Wallet};

use crate::{app::AppMessage, styles};
use types::{address::Address, assets::FungibleAsset};

#[derive(Debug, Clone)]
pub enum Icon {
    None,
    Loading,
    Some(Handle),
}

#[derive(Debug, Clone)]
pub struct FungibleView {
    pub fungible: FungibleAsset,
    pub image: Icon,
}

impl<'a> FungibleView {
    pub fn new(fungible: FungibleAsset, image: Icon) -> Self {
        Self { fungible, image }
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> iced::Element<'a, AppMessage> {
        let Some(resource) = wallet.resources().get(&self.fungible.resource_address) else {
            return container(text("Token not found"))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        };

        let name = text(resource.name.as_str()).size(15).line_height(2.);

        let image: Element<'a, AppMessage> = match &self.image {
            Icon::Some(handle) => widget::image(handle.clone()).into(),
            Icon::Loading => Container::new("").width(150).height(150).into(),
            Icon::None => container(text(Bootstrap::Image).font(BOOTSTRAP_FONT).size(100))
                .center_x(150)
                .center_y(150)
                .into(),
        };

        let amount = row![
            text(&self.fungible.amount)
                .line_height(1.5)
                .size(12)
                .width(Length::Shrink),
            text(resource.symbol.as_str())
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

        let description = text(resource.description.as_str())
            .line_height(1.5)
            .size(12)
            .width(Length::Fill);

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let address = row![
            text("Address").size(12),
            space,
            text(self.fungible.resource_address.truncate()).size(12)
        ]
        .padding(Padding {
            top: 5.,
            ..Padding::from(0)
        });

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let current_supply = row![
            text("Current Supply").size(12),
            space,
            text(resource.current_supply.as_str()).size(12),
        ];

        let divisibility = row![
            text("Divisibility").size(12),
            widget::horizontal_space(),
            text(resource.divisibility.unwrap_or(1)).size(12),
        ];

        let mut tags = row![].spacing(5);

        for tag in resource.tags.iter() {
            tags = tags.push(
                widget::container(widget::text(tag.as_str()).size(12))
                    .center_x(Length::Shrink)
                    .center_y(Length::Shrink)
                    .padding(Padding::new(5.).right(10.))
                    .style(styles::container::tag),
            )
        }

        let tags = column![row![text("Tags").size(12), horizontal_space()], tags].spacing(10);

        let col = widget::column![
            image_name_amount,
            Rule::horizontal(2),
            description,
            Rule::horizontal(2),
            address,
            current_supply,
            divisibility,
            tags
        ]
        .spacing(15)
        .align_x(iced::Alignment::Center)
        .height(Length::Shrink)
        .width(Length::Fill)
        .padding(Padding::from([0, 10]));

        let content = container(col)
            .padding(15)
            .style(styles::container::weak_layer_1_rounded_with_shadow);

        let scrollable =
            widget::scrollable(content).style(styles::scrollable::vertical_scrollable_primary);

        container(scrollable)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .max_width(1000)
            .padding(20)
            .into()
    }
}
