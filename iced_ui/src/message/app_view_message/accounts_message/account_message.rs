use iced::Command;

use crate::{
    app::App,
    message::{app_view_message::AppViewMessage, Message},
    view::app_view::{
        accounts_view::{account_view::AssetView, fungibles_view::FungiblesView, AccountsView},
        ActiveTab,
    },
};
use types::{AccountAddress, Fungible};

use super::{fungibles_message::FungiblesMessage, AccountsViewMessage};

#[derive(Debug, Clone)]
pub enum AccountViewMessage {
    FungiblesView(AccountAddress),
    NonFungiblesView(AccountAddress),
    SelectFungible(Fungible),
    SelectNonFungible {
        account_id: usize,
        non_fungible_id: usize,
    },
    SelectPoolUnit {
        account_id: usize,
        poolunit_id: usize,
    },
    FungiblesMessage(FungiblesMessage),
    //Transaction(Account),
}

impl Into<Message> for AccountViewMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::AccountsViewMessage(
            AccountsViewMessage::AccountViewMessage(self),
        ))
    }
}

impl<'a> AccountViewMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self {
            Self::FungiblesView(account_address) => Self::set_view_fungibles(account_address, app),
            Self::NonFungiblesView(account_address) => {
                Self::set_view_non_fungibles(account_address, app)
            }
            Self::SelectFungible(fungible) => Self::select_fungible(fungible, app),
            Self::SelectNonFungible {
                account_id,
                non_fungible_id,
            } => Self::select_non_fungible(account_id, non_fungible_id, app),
            Self::SelectPoolUnit {
                account_id,
                poolunit_id,
            } => Self::select_poolunit(account_id, poolunit_id, app),
            Self::FungiblesMessage(fungible_message) => fungible_message.process(app),
            // Self::Transaction(account) => Self::transaction_from_account(account, app),
        }
    }

    fn set_view_fungibles(account_address: AccountAddress, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Accounts(AccountsView::Account(ref mut account_view)) =
            app.appview.active_tab
        {
            account_view.view = AssetView::Fungibles(FungiblesView::new(account_address))
        } else {
            unreachable!("{}:{} invalid gui state", module_path!(), line!())
        }

        Command::none()
    }

    fn set_view_non_fungibles(
        account_address: AccountAddress,
        app: &'a mut App,
    ) -> Command<Message> {
        if let ActiveTab::Accounts(AccountsView::Account(ref mut account_view)) =
            app.appview.active_tab
        {
            account_view.view = AssetView::NonFungibles
        } else {
            unreachable!("{}:{} invalid gui state", module_path!(), line!())
        }
        Command::none()
    }

    fn set_view_poolunits(app: &'a mut App) -> Command<Message> {
        // if let AccountsView::Account(mut account_view) = app.appview.center_panel.accounts {
        //     if let View::Fungibles = account_view.view {
        //         account_view.view =
        //     } else {
        //         unreachable!("{}:{} invalid gui state", module_path!(), line!())
        //     }
        // } else {
        //     unreachable!("{}:{} invalid gui state", module_path!(), line!())
        // }
        Command::none()
    }

    fn select_fungible(fungible: Fungible, app: &'a mut App) -> Command<Message> {
        Command::none()
    }

    fn select_non_fungible(
        account_id: usize,
        non_fungible_id: usize,
        app: &'a mut App,
    ) -> Command<Message> {
        Command::none()
    }

    fn select_poolunit(
        account_id: usize,
        poolunit_id: usize,
        app: &'a mut App,
    ) -> Command<Message> {
        Command::none()
    }

    // fn transaction_from_account(account: Account, app: &'a mut App) -> Command<Message> {
    //     if let ActiveTab::Accounts(AccountsView::Account(ref account_view)) =
    //         app.appview.active_tab
    //     {
    //         app.appview.active_tab =
    //             ActiveTab::Transaction(TransactionView::from_account(account))
    //     } else {
    //         unreachable!("{}:{} Invalid state", module_path!(), line!())
    //     }

    //     Command::none()
    // }
}
