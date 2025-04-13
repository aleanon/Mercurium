use deps::*;

use iced::{widget::{self, column, row, scrollable::{Direction, Scrollbar}, Scrollable, Text, TextInput}, Element, Length, Task};
use types::{address::Address, Account};
use wallet::{wallet::Wallet, Setup};

use crate::initial::common::{nav_button, nav_row};

#[derive(Clone)]
pub enum Message {
    Back,
    Next,
    InputAccountName(usize, String),
}

#[derive(Debug)]
pub struct NameAccounts {
    pub accounts: Vec<Account>
}

impl<'a> NameAccounts {
    pub fn new(wallet: &'a Wallet<Setup>) -> Self {
        Self {
            accounts: wallet.selected_accounts(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputAccountName(index, input) => {
                if let Some(account) = self.accounts.get_mut(index) {
                    account.name = input;
                }
            }
            Message::Back | Message::Next => {/*Handled in parent*/}
        }
        
        Task::none()
    }

    pub fn save_to_wallet(&mut self, wallet: &'a mut Wallet<Setup>) {
        wallet.set_accounts(self.accounts.clone());
    }
}


impl<'a> NameAccounts {
    pub fn view(&self) -> Element<'a, Message> {
        let mut accounts = column![];

        for (index, account) in self.accounts.iter().enumerate() {
            let account_truncated = Text::new(account.address.truncate_long())
                .width(Length::Shrink);

            let input_field = TextInput::new("Account name", &account.name)
                .on_input(move |input|Message::InputAccountName(index, input))
                .on_paste(move |input|Message::InputAccountName(index, input))
                .width(Length::Fill);

            let account_row = row![account_truncated, input_field]
                .width(600);

            let account_and_rule = column![account_row, widget::Rule::horizontal(2)]
                .spacing(3);

            accounts = accounts.push(account_and_rule);
        }

        let content = Scrollable::new(accounts)
            .width(Length::Shrink)
            .height(500)
            .spacing(5)
            .direction(Direction::Vertical(Scrollbar::new()));

        let nav = nav_row(
            nav_button("Back", Message::Back),
            nav_button("Next", Message::Next),
        );

        let content_and_nav = column![content, nav];

        widget::container(content_and_nav)
            .center_x(660)
            .center_y(700)
            .into()
    }
}