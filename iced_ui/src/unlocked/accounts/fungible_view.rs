use iced::{
    theme,
    widget::{self, container, image::Handle, row, text},
    Element, Length, Padding,
};
use ravault_iced_theme::styles;

use crate::{app::AppData, app::AppMessage};
use types::assets::FungibleAsset;

const FUNGIBLE_VIEW_WIDTH: Length = Length::Fixed(300.);

#[derive(Debug, Clone)]
pub struct FungibleView {
    pub fungible: FungibleAsset,
    pub image: Option<Handle>,
}

impl<'a> FungibleView {
    pub fn new(fungible: FungibleAsset, image: Option<Handle>) -> Self {
        Self { fungible, image }
    }

    pub fn view(&self, appdata: &'a AppData) -> iced::Element<'a, AppMessage> {
        let resource = appdata.resources.get(&self.fungible.resource_address);

        let name = text(
            resource
                .and_then(|resource| Some(resource.name.as_str()))
                .unwrap_or("NoName"),
        )
        .size(15)
        .line_height(2.);

        let image: Element<'a, AppMessage> = match &self.image {
            Some(handle) => widget::image(handle.clone()).into(),
            None => container(
                text(iced_aw::Bootstrap::Image)
                    .font(iced_aw::BOOTSTRAP_FONT)
                    .size(100),
            )
            .width(150)
            .height(150)
            .center_x()
            .center_y()
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
        .align_items(iced::Alignment::Center);

        let col = widget::column![name, image, amount]
            .align_items(iced::Alignment::Center)
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
            text(&self.fungible.resource_address.truncate()).size(12)
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
            .align_items(iced::Alignment::Center)
            .height(Length::Shrink)
            .width(FUNGIBLE_VIEW_WIDTH)
            .padding(Padding::from([0, 10]));

        let scrollable = widget::scrollable(col)
            .style(theme::Scrollable::custom(styles::scrollable::Scrollable));

        let space_left = widget::Space::new(Length::Fill, Length::Fill);
        let space_right = widget::Space::new(Length::Fill, Length::Fill);

        row![space_left, scrollable, space_right]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
