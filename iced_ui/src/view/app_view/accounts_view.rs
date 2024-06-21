pub mod account_view;
pub mod fungible_view;
pub mod fungibles_view;

use std::collections::{BTreeSet, HashMap};

use crate::{
    app::App,
    message::{
        app_view_message::{accounts_message::AccountsViewMessage, AppViewMessage},
        Message,
    },
};
use iced::{
    theme,
    widget::{self, button, column, container, row, scrollable, text},
    Element, Length,
};
use ravault_iced_theme::styles::{self, button::AccountButton};
use types::{Account, AccountAddress};

use self::account_view::AccountView;

use super::overlay::{add_account_view::AddAccountView, Overlay};

#[derive(Debug, Clone)]
pub enum AccountsView {
    // The hashmap is used to store which accounts are expanded
    OverView(HashMap<AccountAddress, bool>),
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
            Self::OverView(is_expanded) => Self::overview(is_expanded, app),
            Self::Account(account) => account.view(app),
        }
    }

    fn overview(is_expanded: &HashMap<AccountAddress, bool>, app: &'a App) -> Element<'a, Message> {
        let title = text("Accounts").size(25);

        let new_account = button(
            row![
                text(iced_aw::Bootstrap::Plus)
                    .font(iced_aw::BOOTSTRAP_FONT)
                    .size(16),
                text("Account").size(16)
            ]
            .align_items(iced::Alignment::End),
        )
        .style(theme::Button::custom(styles::button::GeneralButton))
        .on_press(AppViewMessage::SpawnOverlay(Overlay::AddAccount(AddAccountView::new())).into());

        let header = row![title, widget::Space::new(Length::Fill, 1), new_account]
            .align_items(iced::Alignment::End)
            .padding(20);

        let accounts = app
            .app_data
            .accounts
            .iter()
            .map(|(_, account)| account)
            .collect::<BTreeSet<&Account>>();

        let mut children: Vec<Element<'a, Message>> = Vec::new();

        for account in accounts {
            let expanded = is_expanded
                .get(&account.address)
                .unwrap_or(&false)
                .to_owned();

            let summary = Self::view_account_summary(expanded, account);

            children.push(summary.into())
        }

        let col = iced::widget::Column::with_children(children)
            .spacing(30)
            // .width(Length::FillPortion(9))
            .padding([0, 15, 15, 0]);

        let scrollable = scrollable::Scrollable::new(col)
            .height(Length::Fill)
            .width(Length::Fill)
            .style(theme::Scrollable::custom(styles::scrollable::Scrollable));
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

    // fn view_account(account: AccountView, app: &'a App) -> Element<'a, Message> {

    //     container(account.view(app))
    //         .width(Length::Fill)
    //         .height(Length::Fill)
    //         .style(Container::Transparent)
    //         .padding([30, 15, 0, 30])
    //         .into()
    // }

    fn view_account_summary(
        _expanded: bool,
        account: &Account,
    ) -> iced::widget::Container<'a, Message> {
        // let account_name = account.;
        // let account_address = account.get_address().truncate();

        let account_name_widget = widget::text(&account.name)
            .horizontal_alignment(iced::alignment::Horizontal::Left)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .size(20);

        let space = widget::Space::new(Length::Fill, Length::Shrink);

        let account_address_widget = widget::text(account.address.truncate())
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
            .on_press(AccountsViewMessage::SelectAccount(account.address.clone()).into());

        container(button).width(Length::Fill).height(Length::Shrink)
    }
}
