use iced::{widget::{self, column, container, row}, Element, Length, Task};
use types::{address::Address, Account, AccountSummary};
use wallet::{Setup, Wallet};

use crate::initial::common::{nav_button, nav_row};

#[derive(Clone)]
pub enum Message {
    Back,
    Next, 
    Accounts(Vec<(Account, AccountSummary)>),
    ToggleAccountSelection(usize),
}

#[derive(Debug)]
pub struct ChooseAccounts {
    pub accounts: Vec<(Account, bool, AccountSummary)>
}

impl<'a> ChooseAccounts {
    pub fn new(wallet: &'a Wallet<Setup>) -> (Self, Task<Message>) {
        let instance = Self {
            accounts: Vec::new()
        };
        
        let task_manager = wallet.task_manager();
        let task = Task::perform(async move {
            task_manager.get_accounts_with_summary().await.unwrap_or(Vec::new())
        }, Message::Accounts);

        (instance, task)
    }

    pub fn update(&mut self, message: Message, wallet: &'a mut Wallet<Setup> ) -> Task<Message> {
        match message {
            Message::Accounts(accounts) => {
                let mut accounts = accounts.into_iter()
                    .map(|(account, account_summary)| {
                        if account_summary.has_summary() {
                            (account, true, account_summary)
                        } else {
                            (account, false, account_summary)
                        }
                    })
                    .collect::<Vec<_>>();
                accounts.sort_by_key(|(_,is_selected,_)| !is_selected);

                self.accounts = accounts;
            }
            Message::ToggleAccountSelection(index) => {
                if let Some(account) = self.accounts.get_mut(index) {
                    account.1 = !account.1
                }
            }
            Message::Back | Message::Next => {/*Handled in parent*/}
        }
        Task::none()
    }

    pub fn save_to_wallet(&mut self, wallet: &'a mut Wallet<Setup>) {
        let accounts = self.accounts
            .iter()
            .map(|(account, _, _)| account.clone())
            .collect();
        
        wallet.set_accounts(accounts);
    }
}



impl<'a> ChooseAccounts {
    pub fn view(&'a self) -> Element<'a, Message> {
        let mut accounts = column!().height(400);

        for (index, (account, is_selected, account_summary)) in
            self.accounts.iter().enumerate()
        {
            let account_address = widget::text(account.address.truncate_long());
            let account_summary = widget::text(account_summary.to_string());

            let is_selected = widget::checkbox("", *is_selected).on_toggle(move |_| {
                Message::ToggleAccountSelection(index)
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
        
        let content = widget::scrollable(accounts.width(400));

        let content_container = container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        
        let nav = nav_row(
            nav_button("Back", Message::Back),
            nav_button("Next", Message::Next),
        );

        let content_and_nav = column![content_container, nav];

        widget::container(content_and_nav)
            .center_x(650)
            .center_y(550)
            .into()
    }
}