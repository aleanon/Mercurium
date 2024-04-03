pub mod fungibles_view;
pub mod fungible_view;

use std::collections::BTreeMap;


use iced::theme;
use iced::widget::button;
use iced::{
    alignment, border::Radius, color, widget::{
        self, column, container, row,
        scrollable::{self, Properties},
        text, Button }, Background, Color, Element, Length, Padding, Vector 
};
use ravault_iced_theme::styles::accounts::AssetListItem;
use crate::app::App;
use crate::message::app_view_message::accounts_message::account_message::AccountViewMessage;
use crate::message::app_view_message::transaction_message::TransactionMessage;
use crate::message::common_message::CommonMessage;
use crate::message::Message;

use store::Db;
use types::{Account, AccountAddress, EntityAccount};

use self::fungibles_view::FungiblesView;


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
    pub fn new(name: String, address: AccountAddress) -> Self {
        Self {
            name,
            address: address.clone(),
            // fungibles: None,
            // non_fungibles: None,
            view: AssetView::Fungibles(FungiblesView::new(address)),
        }
    }

    pub fn from_account(account: &EntityAccount) -> Self {
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
        let db = app.db
            .as_ref()
            .unwrap_or_else(|| unreachable!("{}:{} Database not found", module_path!(), line!()));

        let mut accounts = db.get_accounts_map().unwrap_or(BTreeMap::new());

        let account = accounts.remove(&self.address).unwrap_or(Account::none());
        
        let account_name = text(&self.name)
            .line_height(2.5)
            .size(20)
            .width(Length::Shrink)
            .vertical_alignment(alignment::Vertical::Bottom);

        let account_address = text(self.address.truncate())
            .line_height(1.5)
            .size(15)
            .width(Length::Shrink)
            .vertical_alignment(iced::alignment::Vertical::Bottom);

        let account_address_button = Button::new(account_address)
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

        let transfer_button = Self::nav_button("Send")
            .on_press(TransactionMessage::FromAccount(account).into());

        let receive_button = Self::nav_button("Receive");
        //TODO: On press spawn modal with qr code with accound address and the address written out with a copy button

        let nav_button_row = row![history_button, transfer_button, receive_button]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);

        let nav_button_cont = container(nav_button_row)
            .center_x()
            .width(Length::Fill)
            .height(Length::Shrink);

        let fung_button = Self::select_asset_button("Tokens")
            .on_press(AccountViewMessage::FungiblesView(self.address.clone()).into());

        let nft_button = Self::select_asset_button("NFTs")
            .on_press(AccountViewMessage::NonFungiblesView(self.address.clone()).into());

        let asset_button_row = row![fung_button, nft_button]
            .spacing(100)
            .height(Length::Shrink)
            .width(Length::Shrink);

        let asset_button_cont = container(asset_button_row)
            .center_x()
            .width(Length::Fill)
            .height(Length::Shrink);

        let assets = match self.view {
            AssetView::Fungibles(ref fungibles_view) => fungibles_view.view(app),
            AssetView::NonFungibles => {
                let db = app
                    .db
                    .as_ref()
                    .unwrap_or_else(|| unreachable!("No database"));
                self.view_non_fungibles(db)
            }
        };

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

    pub fn view_non_fungibles(&self, db: &Db) -> iced::Element<'a, Message> {
        let non_fungibles = db
            .get_non_fungibles_by_account(&self.address)
            .unwrap_or_else(|_| None);

        let column = match non_fungibles {
            Some(ref non_fungibles) => {
                //Each non-fungible is turned into an element

                let mut elements: Vec<Element<'a, Message>> = Vec::new();

                for non_fungible in non_fungibles {
                    let icon: iced::Element<'a, Message> = match non_fungible.icon {
                        Some(ref icon) => {
                            widget::image(icon.handle()).width(40).height(40).into()
                        }
                        None => widget::Space::new(40, 40).into(),
                    };

                    let symbol = text(&non_fungible.symbol)
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

                    let container = container(button)
                        .style(AssetListItem::style);

                    elements.push(container.into())
                }

                column(elements)
                    .align_items(iced::Alignment::Center)
                    .padding(Padding {
                        right: 15.,
                        ..Padding::ZERO
                    })
                    .width(Length::Fill)
            }
            None => {
                column![]
            }
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
        // .style(iced::theme::Button::Custom(Box::new(ListButton)))
    }

    pub fn text_color() -> Color {
        color!(255, 255, 255)
    }

    pub fn style(_theme: &iced::Theme) -> container::Appearance {
        container::Appearance {
            //background: Some(iced::Background::Color(Color::TRANSPARENT)),
            border: iced::Border { color: Color::TRANSPARENT, width: 1., radius: Radius::from([0.5;4])},
            shadow: iced::Shadow { color: Color::BLACK, offset: Vector::new(0.5, 0.5), blur_radius: 0.5 },
            background: Some(iced::Background::Color(ListButton::BACKGROUND_ACTIVE_DARK)),
            text_color: Some(AccountView::text_color()),
        }
    }
}

pub struct ListButton;

impl ListButton {
    pub const BACKGROUND_ACTIVE_DARK: Color = Color {
        r: 0.25,
        g: 0.25,
        b: 0.30,
        a: 1.0,
    };

    pub const BACKGROUND_ACTIVE_LIGHT: iced::Color = iced::Color {
        r: 0.92,
        g: 0.92,
        b: 0.92,
        a: 1.0,
    };

    pub const BACKGROUND_HOVERED_DARK: Color = Color {
        r: 0.23,
        g: 0.23,
        b: 0.28,
        a: 1.0,
    };

    pub const BACKGROUND_HOVERED_LIGHT: iced::Color = iced::Color {
        r: 0.93,
        g: 0.93,
        b: 0.93,
        a: 1.0,
    };
}

// impl button::StyleSheet for ListButton {
//     type Style = iced::Theme;
//     fn active(&self, style: &Self::Style) -> button::Appearance {
//         let (background_color, text_color) = match style {
//             iced::Theme::Dark => (Self::BACKGROUND_ACTIVE_DARK, SidePanelButton::text_dark()),
//             iced::Theme::Light => (Self::BACKGROUND_ACTIVE_LIGHT, SidePanelButton::text_light()),
//             _ => (Self::BACKGROUND_ACTIVE_DARK, SidePanelButton::text_dark()),
//         };

//         button::Appearance {
//             background: Some(iced::Background::Color(background_color)),
//             text_color: text_color,
//             border_radius: BorderRadius::from(10.),
//             border_color: background_color,
//             ..Default::default()
//         }
//     }

//     fn hovered(&self, style: &Self::Style) -> button::Appearance {
//         match style {
//             iced::Theme::Dark => button::Appearance {
//                 background: Some(iced::Background::Color(Self::BACKGROUND_HOVERED_DARK)),
//                 ..self.active(style)
//             },
//             iced::Theme::Light => button::Appearance {
//                 background: Some(iced::Background::Color(Self::BACKGROUND_HOVERED_LIGHT)),
//                 ..self.active(style)
//             },
//             _ => button::Appearance::default(),
//         }
//     }
// }

#[derive(Debug, Clone, Default)]
pub struct SidePanelButton;

// impl<'a> SidePanelButton {
//     pub fn render(app: &App, name: &str, message: Message) -> Element<'a, Message> {
//         if app.appview.menu.collapsed {
//             Self::render_compact(app, name, message)
//         } else {
//             Self::render_expanded(app, name, message)
//         }
//     }

//     fn render_compact(_state: &App, name: &str, message: Message) -> Element<'a, Message> {
//         widget::Button::new(
//             widget::text(name)
//                 .size(15)
//                 .vertical_alignment(alignment::Vertical::Center)
//                 .horizontal_alignment(alignment::Horizontal::Left),
//         )
//         .width(Length::Fill)
//         .padding([5, 20])
//         .on_press(message)
//         .style(SidePanelButton::style())
//         .into()
//     }

//     fn render_expanded(_state: &App, name: &str, message: Message) -> Element<'a, Message> {
//         widget::Button::new(
//             widget::text(name)
//                 .size(15)
//                 .vertical_alignment(alignment::Vertical::Center)
//                 .horizontal_alignment(alignment::Horizontal::Left),
//         )
//         .width(Length::Fill)
//         .padding([5, 20])
//         .on_press(message)
//         .style(SidePanelButton::style())
//         .into()
//     }

//     pub fn text_dark() -> Color {
//         color!(180, 180, 180, 1.)
//     }

//     pub fn text_light() -> Color {
//         color!(40, 40, 40, 1.)
//     }

//     pub fn style_dark() -> button::Appearance {
//         button::Appearance {
//             background: Some(Background::Color(Menu::BACKGROUND_DARK)),
//             text_color: SidePanelButton::text_dark(),
//             border_width: 0.,
//             border_color: Menu::BACKGROUND_DARK,
//             border_radius: BorderRadius::from([4.0; 4]),
//             shadow_offset: Vector { x: 0.0, y: 0.0 },
//         }
//     }

//     pub fn style_light() -> button::Appearance {
//         button::Appearance {
//             background: Some(Background::Color(Menu::BACKGROUND_LIGHT)),
//             text_color: SidePanelButton::text_light(),
//             border_width: 0.,
//             border_color: Menu::BACKGROUND_LIGHT,
//             border_radius: BorderRadius::from([4.0; 4]),
//             shadow_offset: Vector { x: 0.0, y: 0.0 },
//         }
//     }

//     pub fn style() -> iced::theme::Button {
//         iced::theme::Button::Custom(Box::new(SidePanelButton))
//     }
// }

// impl button::StyleSheet for SidePanelButton {
//     type Style = iced::Theme;

//     fn active(&self, style: &Self::Style) -> button::Appearance {
//         match style {
//             iced::Theme::Dark => SidePanelButton::style_dark(),
//             iced::Theme::Light => SidePanelButton::style_light(),
//             _ => button::Appearance::default(),
//         }
//     }

//     fn hovered(&self, style: &Self::Style) -> button::Appearance {
//         match style {
//             iced::Theme::Dark => button::Appearance {
//                 text_color: color!(230, 230, 230),
//                 ..SidePanelButton::style_dark()
//             },
//             iced::Theme::Light => button::Appearance {
//                 text_color: color!(20, 20, 20),
//                 ..SidePanelButton::style_light()
//             },
//             _ => button::Appearance::default(),
//         }
//     }
// }
