use deps::*;

use iced::{
    Alignment, Element, Length, Padding, Task,
    widget::{self, column, container, row, text},
};
use std::collections::BTreeSet;
use types::{Account, AppError, Transaction, address::Address};
use wallet::{Unlocked, Wallet};

use crate::{app::AppMessage, styles, unlocked::app_view};

#[derive(Debug, Clone)]
pub enum Message {
    SelectAccount(Account),
    /// Persisted transactions finished loading for the selected account.
    Loaded(Result<BTreeSet<Transaction>, AppError>),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::HistoryViewMessage(self))
    }
}

#[derive(Debug)]
pub struct HistoryView {
    selected: Option<Account>,
    transactions: BTreeSet<Transaction>,
    status: Option<String>,
}

impl<'a> HistoryView {
    pub fn new() -> Self {
        Self {
            selected: None,
            transactions: BTreeSet::new(),
            status: None,
        }
    }

    pub fn update(&mut self, message: Message, wallet: &mut Wallet<Unlocked>) -> Task<AppMessage> {
        match message {
            Message::SelectAccount(account) => {
                let db = wallet.db();
                let address = account.address.clone();
                self.selected = Some(account);
                self.status = Some("Loading…".to_string());
                self.transactions.clear();
                Task::perform(
                    wallet::transaction::read_account_transactions(db, address),
                    |result| Message::Loaded(result).into(),
                )
            }
            Message::Loaded(result) => {
                match result {
                    Ok(transactions) => {
                        self.status = if transactions.is_empty() {
                            Some("No transactions yet — sync to fetch history.".to_string())
                        } else {
                            None
                        };
                        self.transactions = transactions;
                    }
                    Err(err) => self.status = Some(err.to_string()),
                }
                Task::none()
            }
        }
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        let header = text("Transaction history")
            .size(20)
            .width(Length::Fill)
            .center();

        let mut accounts: Vec<&Account> = wallet.accounts().values().collect();
        accounts.sort_unstable_by(|a, b| a.id.cmp(&b.id));

        let picker = widget::pick_list(
            accounts.into_iter().cloned().collect::<Vec<Account>>(),
            self.selected.clone(),
            |account| Message::SelectAccount(account).into(),
        )
        .placeholder("Select account")
        .width(Length::Fill)
        .padding(10);

        let list: Element<'a, AppMessage> = if let Some(status) = &self.status {
            text(status.clone()).style(styles::text::muted).into()
        } else {
            column(self.transactions.iter().rev().map(|tx| {
                container(
                    column![
                        text(tx.transaction_address.truncate_long()).size(13),
                        row![
                            text(tx.timestamp.to_string()).size(11).style(styles::text::muted),
                            widget::space::horizontal(),
                            text(format!("{} changes", tx.balance_changes.len()))
                                .size(11)
                                .style(styles::text::muted),
                        ]
                    ]
                    .spacing(3),
                )
                .padding(10)
                .width(Length::Fill)
                .style(styles::container::weak_layer_2_rounded_with_shadow)
                .into()
            }))
            .spacing(8)
            .into()
        };

        let content = column![header, picker, list]
            .spacing(15)
            .align_x(Alignment::Center)
            .padding(Padding {
                left: 10.,
                right: 15.,
                top: 10.,
                bottom: 10.,
            });

        widget::scrollable(content)
            .style(styles::scrollable::vertical_scrollable_secondary)
            .into()
    }
}
