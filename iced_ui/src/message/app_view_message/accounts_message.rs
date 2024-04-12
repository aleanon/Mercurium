pub mod account_message;
pub mod fungibles_message;

use iced::Command;

use crate::{
    app::App,
    message::Message,
    view::app_view::{
        accounts_view::{account_view::AccountView, AccountsView},
        ActiveTab,
    },
};

use self::account_message::AccountViewMessage;

use super::AppViewMessage;

#[derive(Debug, Clone)]
pub enum AccountsViewMessage {
    Overview,
    NewAccount,
    SelectAccount(AccountView),
    //holds the address of the account to be expanded
    ToggleExpand(String),
    AccountViewMessage(AccountViewMessage),
}

impl Into<Message> for AccountsViewMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::AccountsViewMessage(self))
    }
}

impl<'a> AccountsViewMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self {
            Self::NewAccount => Command::none(),
            Self::Overview => Command::none(),
            Self::SelectAccount(account) => Self::select_account(account, app),
            Self::ToggleExpand(address) => Self::toggle_expand(address, app),
            Self::AccountViewMessage(account_view_message) => account_view_message.process(app),
        }
    }

    fn select_account(account: AccountView, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Accounts(ref mut accounts_view) = app.appview.active_tab {
            *accounts_view = AccountsView::Account(account)
        } else {
            unreachable!("{}:{} Invalid gui state", module_path!(), line!())
        }

        Command::none()
    }

    fn toggle_expand(address: String, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Accounts(AccountsView::OverView(ref mut map)) = app.appview.active_tab {
            map.entry(address)
                .and_modify(|bool| *bool = !*bool)
                .or_insert(true);
        } else {
            unreachable!("{}:{} Invalid gui state", module_path!(), line!())
        }

        Command::none()
    }
}
