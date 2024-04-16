use iced::{
    theme,
    widget::{
        self, column, container,
        image::Handle,
        row,
        scrollable::{self, Properties},
        text, Button,
    },
    Element, Length, Padding,
};

use crate::{
    message::{app_view_message::accounts_message::fungibles_message::FungiblesMessage, Message},
    App,
};
use ravault_iced_theme::styles::{self, button::AssetListButton, container::AssetListItem};
use types::{AccountAddress, Fungible, Fungibles, ResourceAddress};

use super::fungible_view::{self, FungibleView};

#[derive(Debug, Clone)]
pub struct FungiblesView {
    pub(crate) account_addr: AccountAddress,
    pub(crate) selected: Option<ResourceAddress>,
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
        match self.selected {
            Some(ref address) => {
                if let Some(fungible) = app.app_data.db.get_fungible(address).unwrap_or(None) {
                    FungibleView(fungible).view(app)
                } else {
                    // Create a token not found screen
                    column![].into()
                }
            }
            None => {
                let fungibles = app
                    .app_data
                    .db
                    .get_fungibles_by_account(&self.account_addr)
                    .unwrap_or(Fungibles::new());

                let mut elements: Vec<Element<'a, Message>> = Vec::new();

                for fungible in fungibles.0 {
                    let button = Self::fungible_list_button(&fungible, app)
                        .on_press(FungiblesMessage::SelectFungible(fungible.address).into());

                    let button_container = container(button).style(AssetListItem::style);

                    let rule = widget::Rule::horizontal(2);

                    elements.push(column![button_container, rule].into())
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

    fn fungible_list_button(fungible: &Fungible, app: &'a App) -> Button<'a, Message> {
        let icon: iced::Element<'a, Message> =
            match app.appview.resource_icons.get(&fungible.address) {
                Some(handle) => widget::image(handle.clone()).width(40).height(40).into(),
                None => container(
                    text(iced_aw::BootstrapIcon::Image)
                        .font(iced_aw::BOOTSTRAP_FONT)
                        .size(30),
                )
                .width(40)
                .height(40)
                .center_x()
                .center_y()
                .into(),
            };

        let name_and_symbol = column![
            text(&fungible.name).size(16),
            text(&fungible.symbol).size(14)
        ]
        .spacing(3)
        .align_items(iced::Alignment::Start);

        let list_button_content = row![
            icon,
            name_and_symbol,
            widget::Space::new(Length::Fill, 1),
            text(format!("{} {}", &fungible.amount, &fungible.symbol)).size(18)
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
