use deps::{iced::widget::image::Handle, *};

use std::collections::{BTreeSet, HashMap};

use crate::{
    app::AppMessage,
    styles,
    unlocked::{app_view, overlays::overlay::SpawnOverlay},
};
use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, button, column, container, row, scrollable, text},
    Element, Length, Padding, Task,
};
use types::{
    address::{AccountAddress, Address},
    Account,
};
use wallet::{Unlocked, Wallet};

use super::account_view::{self, AccountView};

#[derive(Debug, Clone)]
pub enum Message {
    Overview,
    NewAccount,
    SelectAccount(AccountAddress),
    //holds the address of the account to be expanded
    ToggleExpand(AccountAddress),
    AccountViewMessage(account_view::Message),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::AccountsViewMessage(self))
    }
}

#[derive(Debug, Clone)]
pub enum AccountsView {
    // The hashmap is used to store which accounts are expanded
    OverView(HashMap<AccountAddress, bool>),
    Account(AccountView),
}

impl<'a> AccountsView {
    pub fn new() -> Self {
        Self::OverView(HashMap::new())
    }

    pub fn update(
        &mut self,
        message: Message,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        let mut command = Task::none();
        match message {
            Message::NewAccount => {}
            Message::Overview => {}
            Message::SelectAccount(account) => self.select_account(account, wallet),
            Message::ToggleExpand(address) => self.toggle_expand(address),
            Message::AccountViewMessage(account_view_message) => {
                if let Self::Account(account_view) = self {
                    command = account_view.update(account_view_message, wallet)
                }
            }
        }
        command
    }

    fn select_account(
        &mut self,
        account_address: AccountAddress,
        wallet: &'a mut Wallet<Unlocked>,
    ) {
        if let Some(account) = wallet.accounts().get(&account_address) {
            *self = AccountsView::Account(AccountView::from_account(account));
        }
    }

    fn toggle_expand(&mut self, address: AccountAddress) {
        if let Self::OverView(map) = self {
            map.entry(address)
                .and_modify(|bool| *bool = !*bool)
                .or_insert(true);
        }
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        match self {
            Self::OverView(is_expanded) => Self::overview(is_expanded, wallet),
            Self::Account(account) => account.view(wallet),
        }
    }

    fn overview(
        is_expanded: &HashMap<AccountAddress, bool>,
        wallet: &'a Wallet<Unlocked>,
    ) -> Element<'a, AppMessage> {
        let title = text("Accounts").size(25);

        let new_account = button(
            row![
                text(Bootstrap::Plus).font(BOOTSTRAP_FONT).size(16),
                text("Add Account").size(16)
            ]
            .align_y(iced::Alignment::End),
        )
        .style(styles::button::base_layer_2_rounded_with_shadow)
        .on_press(app_view::Message::SpawnOverlay(SpawnOverlay::AddAccount).into());

        let header = row![title, widget::Space::new(Length::Fill, 1), new_account]
            .align_y(iced::Alignment::End)
            .padding(20);

        let accounts = wallet
            .accounts()
            .iter()
            .map(|(_, account)| account)
            .collect::<BTreeSet<&Account>>();

        let mut children: Vec<Element<'a, AppMessage>> = Vec::new();

        for account in accounts {
            let expanded = is_expanded
                .get(&account.address)
                .unwrap_or(&false)
                .to_owned();

            let summary = Self::view_account_summary(expanded, account, wallet);

            children.push(summary.into())
        }

        let col = iced::widget::Column::with_children(children)
            .spacing(15)
            // .width(Length::FillPortion(9))
            .padding(Padding {
                bottom: 0.,
                top: 15.,
                right: 15.,
                left: 10.,
            });

        let scrollable = scrollable::Scrollable::new(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .style(styles::scrollable::vertical_scrollable_secondary);
        // .direction(scrollable::Direction::Vertical(Properties::default()));

        let content = widget::column![header, scrollable]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_account_summary(
        _expanded: bool,
        account: &'a Account,
        wallet: &'a Wallet<Unlocked>,
    ) -> iced::widget::Container<'a, AppMessage> {
        let account_name_widget = widget::text(&account.name)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center)
            .size(20);

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let account_address_widget = widget::text(account.address.truncate_long())
            .size(18)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom);

        let name_address_row = row![account_name_widget, space, account_address_widget];

        let mut icons: Vec<Element<AppMessage>> = Vec::new();

        let fungibles = wallet
            .fungibles()
            .get(&account.address)
            .and_then(|fungibles| {
                for fungible in fungibles.iter() {
                    if icons.len() >= 6 {
                        break;
                    };

                    let icon = wallet
                        .resource_icons()
                        .get(&fungible.resource_address)
                        .and_then(|bytes| Some(Handle::from_bytes(bytes.clone())));

                    if let Some(handle) = icon {
                        icons.push(widget::image(handle).width(20).height(20).into());
                    }
                }
                Some(fungibles.len())
            })
            .unwrap_or(0);

        let non_fungibles = wallet
            .non_fungibles()
            .get(&account.address)
            .and_then(|non_fungibles| {
                for non_fungible in non_fungibles {
                    if icons.len() >= 6 {
                        break;
                    };

                    let icon = wallet
                        .resource_icons()
                        .get(&non_fungible.resource_address)
                        .and_then(|bytes| Some(Handle::from_bytes(bytes.clone())));

                    if let Some(handle) = icon {
                        icons.push(widget::image(handle).width(20).height(20).into());
                    }
                }
                Some(non_fungibles.len())
            })
            .unwrap_or(0);

        let not_showing = fungibles + non_fungibles - icons.len();

        let mut icons = row(icons).spacing(5);

        if not_showing > 0 {
            icons = icons.push(widget::Space::new(5, 1));
            icons = icons.push(text!("+ {}", not_showing))
        }

        let space = iced::widget::Space::new(Length::Fill, Length::Fill);

        let columns = column![name_address_row, icons].spacing(20);

        let button = widget::button(columns)
            .height(100)
            .width(Length::Fill)
            .style(styles::button::base_layer_2_rounded_with_shadow)
            .padding(15)
            .on_press(Message::SelectAccount(account.address.clone()).into());

        container(button)
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding(5)
    }
}
