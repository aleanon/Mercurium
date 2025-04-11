use super::{name_accounts::NameAccounts, set_password::SetPassword};
use bip39::Mnemonic;
use iced::{
    widget::{self, column, row, Text}, Element, Length, Task
};
use types::{address::Address, crypto::Password, Account, AccountSummary, AppError};
use wallet::Wallet;

use crate::{
    app::AppMessage,
    initial::{restore_from_seed, setup},
};


#[derive(Debug, Clone)]
pub enum Message {
    ToggleAccountSelection(usize),
    SetAccountsStartIndex(usize),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(
            setup::Message::RestoreFromSeedMessage(
                restore_from_seed::Message::ChooseAccountMessage(self)))
    }
}


#[derive(Debug)]
pub struct ChooseAccounts {
    pub notification: &'static str,
    pub mnemonic: Mnemonic,
    pub seed_password: Option<Password>,
    pub password: Password,
    pub show_from_current_index: usize,
}

impl ChooseAccounts {
    

    const ACCOUNTS_PER_PAGE: usize = 20;

    pub fn from_page_set_password(page: SetPassword) -> Self {
        Self {
            notification: "",
            mnemonic: page.mnemonic,
            seed_password: page.seed_password,
            password: page.password,
            show_from_current_index: 0,
        }
    }
    
    pub fn from_page_name_accounts(name_accounts: NameAccounts) -> ChooseAccounts {
        Self {
            notification: "",
            mnemonic: name_accounts.mnemonic,
            seed_password: name_accounts.seed_password,
            password: name_accounts.password,
            show_from_current_index: 0,
        }
    }    
}


impl<'a> ChooseAccounts {
    pub fn update(&mut self, message: Message, accounts: Option<&'a mut Vec<(Account, bool, AccountSummary)>>) -> Result<Task<AppMessage>, AppError> {
        match message {
            Message::ToggleAccountSelection(account_index) => self.toggle_account_selection(account_index, accounts),
            Message::SetAccountsStartIndex(start_index) => self.show_from_current_index = start_index,
        }

        Ok(Task::none())
    }
    
    fn toggle_account_selection(&mut self, account_index: usize, accounts: Option<&'a mut Vec<(Account, bool, AccountSummary)>>) {
        let Some(accounts) = accounts else {return};
        let Some((_, is_selected, _)) = accounts.get_mut(account_index) else {return};
            
        *is_selected =!*is_selected
    }


    pub fn view(&'a self, accounts_data:Option<&Vec<(Account, bool, AccountSummary)>> ) -> Element<'a, AppMessage> {
        let mut accounts = column!().height(400);

        let start_index = self.show_from_current_index;
        let accounts_data = match accounts_data{
            Some(data) => data, 
            None => {
                return Text::new("Waiting for account generation...").into()
            },
        };

        if let Some(accounts_selection) = accounts_data.get(start_index..start_index + Self::ACCOUNTS_PER_PAGE) {

            for (i, (account, is_selected, account_summary)) in
                accounts_selection.iter().enumerate()
            {
                let account_address = widget::text(account.address.truncate_long());
                let account_summary = widget::text(account_summary.to_string());

                let is_selected = widget::checkbox("", *is_selected).on_toggle(move |_| {
                    Message::ToggleAccountSelection(start_index + i).into()
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
        } else {
            accounts = accounts.push(
                Text::new("No more accounts found")
            )
        };

        let row = row![
            widget::button("Previous Page").on_press_maybe(self.previous_accounts_chunk()),
            accounts.width(400),
            widget::button("Next Page").on_press(self.next_accounts_chunk())
        ]
        .align_y(iced::Alignment::Center);



        widget::column![row]
            .align_x(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
            .into()
    }


    fn previous_accounts_chunk(&self) -> Option<AppMessage> {
        if self.show_from_current_index == 0 {
            None
        } else {
            Some(Message::SetAccountsStartIndex(self.show_from_current_index - Self::ACCOUNTS_PER_PAGE).into())
        }
    }

    
    fn next_accounts_chunk(&self) -> AppMessage {
        // TODO: generer flere kontoer om det er mindre enn tre sider å vise

        Message::SetAccountsStartIndex(self.show_from_current_index + Self::ACCOUNTS_PER_PAGE).into()
    }
}
