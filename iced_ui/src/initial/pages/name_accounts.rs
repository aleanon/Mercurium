use bip39::Mnemonic;
use iced::{widget::{self, column, row, scrollable::{Direction, Scrollbar}, Scrollable, Text, TextInput}, Element, Length, Task};
use types::{address::Address, crypto::Password, Account, AccountSummary, AppError};

use crate::{app::AppMessage, initial::{restore_from_seed, setup}};

use super::choose_account::ChooseAccounts;


#[derive(Debug, Clone)]
pub enum Message {
    NameInput(usize, String)
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(
            setup::Message::RestoreFromSeedMessage(
                restore_from_seed::Message::NameAccountsMessage(self)
            )
        )
    }
}

#[derive(Debug)]
pub struct NameAccounts {
    pub notification: &'static str,
    pub mnemonic: Mnemonic,
    pub seed_password: Option<Password>,
    pub password: Password,
    pub accounts: Vec<Account>,
}

impl NameAccounts {

    pub fn from_page_choose_accounts(page: ChooseAccounts, accounts: Option<&Vec<(Account, bool, AccountSummary)>>) -> Self {
        let accounts = match accounts {
            Some(accounts) => accounts,
            None => &Vec::new(),
        };

        let accounts = accounts
            .iter()
            .filter_map(|(account, is_selected, _)| {
                if *is_selected {Some(account.clone())} else {None}
            })
            .collect();

        Self {
            notification: "",
            mnemonic: page.mnemonic,
            seed_password: page.seed_password,
            password: page.password,
            accounts,
        }
    }
    
}

impl<'a> NameAccounts {
    pub fn update(&mut self, message: Message) -> Result<Task<AppMessage>, AppError> {
        match message {
            Message::NameInput(index, input) => self.input_account_name(index, input)
        }
        Ok(Task::none())
    }

    fn input_account_name(&mut self, index: usize, input: String) {
        if let Some(account) = self.accounts.get_mut(index) {
            account.name = input
        }
    }
}


impl<'a> NameAccounts {
    pub fn view(&self) -> Element<'a, AppMessage> {
        let mut accounts = column![];

        for (index, account) in self.accounts.iter().enumerate() {
            let account_truncated = Text::new(account.address.truncate_long())
                .width(Length::Shrink);

            let input_field = TextInput::new("Account name", &account.name)
                .on_input(move |input|Message::NameInput(index, input).into())
                .on_paste(move |input|Message::NameInput(index, input).into())
                .width(Length::Fill);

            let account_row = row![account_truncated, input_field]
                .width(600);

            let account_and_rule = column![account_row, widget::Rule::horizontal(2)]
                .spacing(3);

            accounts = accounts.push(account_and_rule);
        }

        Scrollable::new(accounts)
            .width(Length::Shrink)
            .height(500)
            .spacing(5)
            .direction(Direction::Vertical(Scrollbar::new()))
            .into()
    }
}