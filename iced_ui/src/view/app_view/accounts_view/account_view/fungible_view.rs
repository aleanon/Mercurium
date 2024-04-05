use std::ops::Deref;

use iced::{
    widget::{self, image::Handle, row, svg, text },
    Length, Padding,
};

use crate::{app::App, message::Message};
use types::{Fungible, AppPath};


const EMPTY_IMAGE: &'static [u8; 1000] = &[255; 1000];
const FUNGIBLE_VIEW_WIDTH: Length = Length::Fixed(300.);
pub const NO_IMAGE_ICON: &'static [u8] = include_bytes!("../../../../../../icons/icons8-image-96.png");

pub struct FungibleView(pub Fungible);

impl Deref for FungibleView {
    type Target = Fungible;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



impl<'a> FungibleView {
    pub fn view(&self, app: &'a App) -> iced::Element<'a, Message> {
        let name = if self.name.len() != 0 {
            &self.name
        } else {
            "NoName"
        };

        let name = text(name).size(15).line_height(2.);

        let image = match self.icon {
            Some(_) => {
                let mut icon_path = AppPath::new().unwrap().icons_directory().clone();
                icon_path.push(self.address.as_str());
                icon_path.set_extension("png");
                let handle = Handle::from_path(icon_path);
                widget::image(handle).height(150).width(150)
            }
            None => widget::image(Handle::from_memory(NO_IMAGE_ICON)).height(150).width(150),
        };

        let amount = row![
            text(&self.amount)
                .line_height(1.5)
                .size(12)
                .width(Length::Shrink),
            text(&self.symbol)
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

        let description = text(self.description.as_deref().unwrap_or("No description"))
            .line_height(1.5)
            .size(12)
            .width(Length::Fill);

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let address = row![
            text("Address:").size(12),
            space,
            text(&self.address.truncate()).size(12)
        ]
        .padding(Padding {
            top: 5.,
            ..Padding::from(0)
        });

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let current_supply = row![
            text("Current Supply:").size(12),
            space,
            text(&self.current_supply).size(12),
        ];

        let rule = widget::Rule::horizontal(2);
        let rule2 = widget::Rule::horizontal(2);

        let col = widget::column![col, rule, description, rule2, address, current_supply,]
            .spacing(15)
            .align_items(iced::Alignment::Center)
            .height(Length::Shrink)
            .width(FUNGIBLE_VIEW_WIDTH)
            .padding(Padding::from([0, 10]));

        let scrollable = widget::scrollable(col);

        let space_left = widget::Space::new(Length::Fill, Length::Fill);
        let space_right = widget::Space::new(Length::Fill, Length::Fill);

        row![space_left, scrollable, space_right]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
