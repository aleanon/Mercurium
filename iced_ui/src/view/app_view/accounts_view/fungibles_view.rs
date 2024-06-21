use std::collections::BTreeSet;

use iced::{
    theme,
    widget::{self, column, container, row, text, Button},
    Element, Length, Padding,
};

use crate::{
    message::{app_view_message::accounts_message::fungibles_message::FungiblesMessage, Message},
    App,
};
use ravault_iced_theme::styles::{self, button::AssetListButton, container::AssetListItem};
use types::{assets::FungibleAsset, AccountAddress, Fungible, Fungibles, ResourceAddress};

use super::fungible_view::FungibleView;

#[derive(Debug, Clone)]
pub struct FungiblesView {
    pub account_addr: AccountAddress,
    pub selected: Option<FungibleView>,
}

impl<'a> FungiblesView {
    pub fn new(account_addr: AccountAddress) -> Self {
        Self {
            account_addr,
            selected: None,
        }
    }
}

impl<'a> FungiblesView {
    pub fn view(&self, app: &'a App) -> iced::Element<'a, Message> {
        match &self.selected {
            Some(fungible_view) => fungible_view.view(app),
            None => {
                let mut elements: Vec<Element<'a, Message>> = Vec::new();

                if let Some(fungibles) = app.app_data.fungibles.get(&self.account_addr) {
                    for fungible in fungibles {
                        let button = Self::fungible_list_button(fungible, app)
                            .on_press(FungiblesMessage::SelectFungible(fungible.clone()).into());

                        let button_container = container(button).style(AssetListItem::style);

                        let rule = widget::Rule::horizontal(2);

                        elements.push(column![button_container, rule].into())
                    }
                } else {
                    // Push no elements found widget to "elements"
                }

                let column = column(elements)
                    .align_items(iced::Alignment::Center)
                    .padding(Padding {
                        right: 15.,
                        ..Padding::ZERO
                    })
                    .width(Length::Fill);

                widget::scrollable(column)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .style(theme::Scrollable::custom(styles::scrollable::Scrollable))
                    .into()
            }
        }
    }

    fn fungible_list_button(fungible: &FungibleAsset, app: &'a App) -> Button<'a, Message> {
        let icon: iced::Element<'a, Message> =
            match app.app_data.resource_icons.get(&fungible.resource_address) {
                Some(handle) => widget::image(handle.clone()).width(40).height(40).into(),
                None => container(
                    text(iced_aw::Bootstrap::Image)
                        .font(iced_aw::BOOTSTRAP_FONT)
                        .size(30),
                )
                .width(40)
                .height(40)
                .center_x()
                .center_y()
                .into(),
            };
        let (name, symbol) = match app.app_data.resources.get(&fungible.resource_address) {
            Some(resource) => (resource.name.as_str(), resource.symbol.as_str()),
            None => ("NoName", ""),
        };

        let name_and_symbol = column![text(name).size(16), text(&symbol).size(14)]
            .spacing(3)
            .align_items(iced::Alignment::Start);

        let list_button_content = row![
            icon,
            name_and_symbol,
            widget::Space::new(Length::Fill, 1),
            text(format!("{} {}", &fungible.amount, symbol)).size(18)
        ]
        .padding(Padding {
            left: 10.,
            right: 10.,
            bottom: 5.,
            top: 5.,
        })
        .spacing(15)
        .align_items(iced::Alignment::Center);

        widget::button(list_button_content).style(theme::Button::custom(AssetListButton))
    }
}
