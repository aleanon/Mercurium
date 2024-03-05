pub mod account_view;

use std::collections::HashMap;

use iced::{
    widget::{
        self, column, container, row,
        scrollable::{self, Properties},
    },
    Element, Length,
};

use crate::{app::App, message::{app_view_message::accounts_message::AccountsViewMessage, Message}};
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
        let db = app
            .db
            .as_ref()
            .unwrap_or_else(|| unreachable!("No database found"));

        let accounts = db.get_entityaccounts().unwrap_or_else(|_| Vec::new());

        let header = widget::text("Accounts")
            .size(25)
            .line_height(3.)
            .width(Length::Fill);

        let mut children:Vec<Element<'a, Message>> = Vec::new();

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
        //assert!(self.account_id == account.get_id(), "Account id mismatch, in render account");
        let account_name = account.get_name();
        let account_address = account.get_address().truncate();
        let account_id = account.get_id();
        //let account_state = self.accounts.get(&account_id).expect("Account state not found, should be unreachable");


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

        let account_address_button = widget::button(name_address_row)
            .height(Length::Shrink)
            .width(Length::Fill)
            .on_press(AccountsViewMessage::SelectAccount(AccountView::from_account(account)).into());

        let name_address_container = container(account_address_button).width(Length::Fill).height(Length::Shrink);

        let space = iced::widget::Space::new(Length::Fill, Length::FillPortion(2));

        let expand = widget::button(widget::text("exp").size(5))
            .height(10)
            .width(20)
            .on_press(AccountsViewMessage::ToggleExpand(account.name.clone()).into());

        let expand_container = container(expand)
            .align_x(iced::alignment::Horizontal::Right)
            .height(Length::Shrink)
            .width(Length::Fill)
            .padding([0, 10, 0, 0]);

        let columns;
        let height;

        if expanded {
            let (token_rule, allow_specific, deny_specific, allow_depositor) =
                account.deposit_rules().show_rules();

            let first_row = row![
                widget::text("Token Rule:")
                    .size(10)
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
                widget::text(format!("{token_rule}"))
                    .size(10)
                    .horizontal_alignment(iced::alignment::Horizontal::Right)
                    .width(Length::Fill)
            ]
            .width(Length::Fill)
            .padding(5);

            let second_row = row![
                widget::text("Allow Specific:")
                    .size(10)
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
                widget::text(if let Some(_) = allow_specific {
                    "Rule Set: Watch in SETTINGS"
                } else {
                    "No Rule Set"
                })
                .size(10)
                .horizontal_alignment(iced::alignment::Horizontal::Right)
                .width(Length::Fill),
            ]
            .width(Length::Fill)
            .padding(5);

            let third_row = row![
                widget::text("Deny Specific:")
                    .size(10)
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
                widget::text(if let Some(_) = deny_specific {
                    "Rule Set: Watch in SETTINGS"
                } else {
                    "No Rule Set"
                })
                .size(10)
                .horizontal_alignment(iced::alignment::Horizontal::Right)
                .width(Length::Fill),
            ]
            .width(Length::Fill)
            .padding(5);

            let fourth_row = row![
                widget::text("Allow Depositor:")
                    .size(10)
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
                widget::text(if let Some(_) = allow_depositor {
                    "Rule Set: Watch in SETTINGS"
                } else {
                    "No Rule Set"
                })
                .size(10)
                .horizontal_alignment(iced::alignment::Horizontal::Right)
                .width(Length::Fill),
            ]
            .width(Length::Fill)
            .padding(5);

            columns = column![
                name_address_container,
                space,
                first_row,
                second_row,
                third_row,
                fourth_row,
                expand_container
            ];
            height = 200;
        } else {
            columns = column![name_address_container, space, expand_container];
            height = 100;
        }

        container(columns)
            .width(Length::Fill)
            .height(height)
            .style(AccountView::style)
            .padding(5)
    }
}
