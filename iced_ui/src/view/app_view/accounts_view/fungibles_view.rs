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
use ravault_iced_theme::styles::{
    self,
    accounts::{AssetListButton, AssetListItem},
};
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
                    .direction(scrollable::Direction::Vertical(Properties::default()))
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
                Some(handle) => widget::image(handle.clone()).width(50).height(50).into(),
                None => widget::image(Handle::from_memory(fungible_view::NO_IMAGE_ICON))
                    .width(50)
                    .height(50)
                    .into(),
            };

        let symbol = match fungible.symbol.len() {
            0 => {
                if fungible.name.len() != 0 {
                    &fungible.name
                } else {
                    "NoName"
                }
            }
            _ => &fungible.symbol,
        };

        let symbol = text(symbol)
            .size(20)
            .height(30)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .horizontal_alignment(iced::alignment::Horizontal::Left)
            .width(Length::Fill);

        let amount = text(&fungible.amount)
            .size(20)
            .height(30)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .horizontal_alignment(iced::alignment::Horizontal::Right)
            .width(Length::Shrink);

        let row = row![icon, symbol, amount]
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(Padding {
                left: 10.,
                right: 10.,
                bottom: 5.,
                top: 5.,
            })
            .spacing(15)
            .align_items(iced::Alignment::Center);

        widget::button(row)
            .width(Length::Fill)
            .height(85)
            .style(theme::Button::custom(AssetListButton))
    }
}
