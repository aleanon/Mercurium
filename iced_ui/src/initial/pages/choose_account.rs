use super::set_password::SetPassword;
use bip39::Mnemonic;
use iced::{
    widget::{self, column, row, Column},
    Length,
};
use types::{address::Address, crypto::Password, Account};

use crate::{
    app::AppMessage,
    initial::{common::{nav_button, nav_row}, restore_from_seed::AccountSummary},
};


#[derive(Debug, Clone)]
pub enum Message {
    ToggleAccountSelection,
}


#[derive(Debug)]
pub struct ChooseAccounts {
    pub notification: &'static str,
    pub mnemonic: Mnemonic,
    pub seed_password: Option<Password>,
    pub password: Password,
    pub accounts: Vec<(Account, bool, AccountSummary)>,
    pub show_from_current_index: usize,
}

impl ChooseAccounts {
    pub fn from_page_set_password(page: SetPassword) -> Self {
        Self {
            notification: "",
            mnemonic: page.mnemonic,
            seed_password: page.seed_password,
            password: page.password,
            accounts: Vec::new(),
            show_from_current_index: 0,
        }
    }


    pub fn update_account_selected(&mut self, account_index: usize) {
        if let Some((_, is_selected, _)) = self.accounts.get_mut(account_index) {
            *is_selected =!*is_selected
        };
    }

    pub fn goto_page_name_accounts(&mut self) {
        self.accounts_data.selected_accounts = self
            .accounts_data
            .accounts
            .iter()
            .flatten()
            .filter_map(|(account, selected, _)| selected.then_some(account.clone()))
            .collect();

        self.stage = Stage::NameAccounts;
    }
}


impl<'a> ChooseAccounts {
    pub fn choose_accounts_view(&'a self) -> Column<'a, AppMessage> {
        let mut accounts = column!().height(400);

        let accounts_pr_view = 20;
        let start_index = self.show_from_current_index;
        let show_accounts = self
            .accounts
            .get(start_index..start_index + accounts_pr_view);

        let Some(accounts_selection) = show_accounts else {return accounts};

        for (i, (account, is_selected, account_summary)) in
            accounts_selection.iter().enumerate()
        {
            let account_address = widget::text(account.address.truncate());
            let account_summary = widget::text(account_summary.to_string());

            let is_selected = widget::checkbox("", *is_selected).on_toggle(move |_| {
                Message::ToggleAccountSelection((self.accounts_data.page_index, i)).into()
            });

            accounts = accounts.push(
                row![
                    account_address.width(Length::FillPortion(10)),
                    widget::Space::new(Length::Fill, 1),
                    account_summary.width(Length::FillPortion(10)),
                    widget::Space::new(Length::FillPortion(2), 1),
                    is_selected.width(Length::FillPortion(2))
                ]
                .width(Length::Fill),
            )
        }
        

        let row = row![
            widget::button("Previous Page").on_press_maybe(if self.accounts_data.page_index == 0 {
                None
            } else {
                Some(Message::NewPage(self.accounts_data.page_index - 1).into())
            }),
            accounts.width(400),
            widget::button("Next Page")
                .on_press(Message::NewPage(self.accounts_data.page_index + 1).into())
        ]
        .align_y(iced::Alignment::Center);

        let nav_buttons = nav_row(
            nav_button("Back").on_press(Message::Back.into()),
            nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![row, nav_buttons]
            .align_x(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }
}
