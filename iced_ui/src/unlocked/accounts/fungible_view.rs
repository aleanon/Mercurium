use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, container, image::Handle, row, text, Container},
    Element, Length, Padding,
};
use wallet::{Unlocked, Wallet};

use crate::{app::AppData, app::AppMessage};
use types::{address::Address, assets::FungibleAsset};

const FUNGIBLE_VIEW_WIDTH: Length = Length::Fixed(300.);

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
        let resource = wallet.wallet_data().resource_data.resources.get(&self.fungible.resource_address);

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
        //     let directory = AppPath::get().icons_directory();
        //     let mut icon_path = directory.clone();
        //     icon_path.push(&self.fungible.address.to_string());
        //     icon_path.set_extension("png");
        //     if icon_path.exists() {
        //         widget::image(Handle::from_path(icon_path))
        //             .width(150)
        //             .height(150)
        //             .into()
        //     } else {
        //         container(
        //             text(iced_aw::Bootstrap::Image)
        //                 .font(iced_aw::BOOTSTRAP_FONT)
        //                 .size(100),
        //         )
        //         .width(150)
        //         .height(150)
        //         .center_x()
        //         .center_y()
        //         .into()
        //     }
        // };

        let amount = row![
            text(&self.fungible.amount)
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

        let col = widget::column![name, image, amount]
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
            text(self.fungible.resource_address.truncate()).size(12)
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

        let col = widget::column![col, rule, description, rule2, address, current_supply,]
            .spacing(15)
            .align_x(iced::Alignment::Center)
            .height(Length::Shrink)
            .width(FUNGIBLE_VIEW_WIDTH)
            .padding(Padding::from([0, 10]));

        let scrollable = widget::scrollable(col).style(styles::scrollable::vertical_scrollable);

        let space_left = widget::Space::new(Length::Fill, Length::Fill);
        let space_right = widget::Space::new(Length::Fill, Length::Fill);

        row![space_left, scrollable, space_right]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
