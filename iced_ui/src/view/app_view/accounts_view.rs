pub mod account_view;
pub mod fungible_view;
pub mod fungibles_view;

use std::collections::HashMap;

use crate::{
    app::App,
    message::{app_view_message::accounts_message::AccountsViewMessage, Message},
};
use iced::{
    theme,
    widget::{
        self, column, container, row,
        scrollable::{self, Properties},
    },
    Element, Length,
};
use ravault_iced_theme::styles::{self, accounts::AccountButton};
use types::EntityAccount;

use self::account_view::AccountView;

#[derive(Debug, Clone)]
pub enum AccountsView {
    OverView(HashMap<String, bool>),
    Account(AccountView),
}

impl AccountsView {
    pub fn new() -> Self {
        Self::OverView(HashMap::new())
    }
}

impl<'a> AccountsView {
    pub fn view(&self, app: &'a App) -> Element<'a, Message> {
        match self {
            Self::OverView(map) => self.overview(map, app),
            Self::Account(account) => account.view(app),
        }
    }

    pub fn overview(&self, map: &HashMap<String, bool>, app: &'a App) -> Element<'a, Message> {
        let accounts = app
            .app_data
            .db
            .get_entityaccounts()
            .unwrap_or_else(|_| Vec::new());

        let header = widget::text("Accounts")
            .size(25)
            .line_height(3.)
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center);

        let mut children: Vec<Element<'a, Message>> = Vec::new();

        for account in accounts.iter() {
            let expanded = map.get(&account.name).unwrap_or(&false).to_owned();

            let summary = self.view_account_summary(expanded, account);

            children.push(summary.into())
        }

        let col = iced::widget::Column::with_children(children)
            .spacing(30)
            .width(Length::FillPortion(9))
            .padding([0, 15, 15, 0]);

        let scrollable = scrollable::Scrollable::new(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .style(theme::Scrollable::custom(styles::scrollable::Scrollable))
            .direction(scrollable::Direction::Vertical(Properties::default()));

        let content = widget::column![header, scrollable]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // fn view_account(account: AccountView, app: &'a App) -> Element<'a, Message> {

    //     container(account.view(app))
    //         .width(Length::Fill)
    //         .height(Length::Fill)
    //         .style(Container::Transparent)
    //         .padding([30, 15, 0, 30])
    //         .into()
    // }

    pub fn view_account_summary(
        &self,
        expanded: bool,
        account: &EntityAccount,
    ) -> iced::widget::Container<'a, Message> {
        let account_name = account.get_name();
        let account_address = account.get_address().truncate();

        let account_name_widget = widget::text(account_name)
            .horizontal_alignment(iced::alignment::Horizontal::Left)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .size(20);

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let account_address_widget = widget::text(account_address)
            .size(18)
            .horizontal_alignment(iced::alignment::Horizontal::Right)
            .vertical_alignment(iced::alignment::Vertical::Bottom);

        let name_address_row = row![account_name_widget, space, account_address_widget];

        let space = iced::widget::Space::new(Length::Fill, Length::Fill);

        let columns = column![name_address_row, space];
        let button = widget::button(columns)
            .height(100)
            .width(Length::Fill)
            .style(theme::Button::custom(AccountButton))
            .padding(5)
            .on_press(
                AccountsViewMessage::SelectAccount(AccountView::from_account(account)).into(),
            );

        container(button).width(Length::Fill).height(Length::Shrink)
    }
}
