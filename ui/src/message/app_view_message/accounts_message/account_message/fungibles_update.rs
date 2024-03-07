use iced::Command;


use types::ResourceAddress;

use crate::{message::{app_view_message::{accounts_message::AccountsViewMessage, AppViewMessage}, Message}, view::app_view::{accounts_view::{account_view::AssetView, AccountsView}, ActiveTab}, App};

use super::AccountViewMessage;

#[derive(Debug, Clone)]
pub enum FungiblesMessage {
    Back,
    SelectFungible(ResourceAddress),
}

impl Into<Message> for FungiblesMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::AccountsViewMessage(
            AccountsViewMessage::AccountViewMessage(AccountViewMessage::FungiblesMessage(self)),
        ))
    }
}

impl<'a> FungiblesMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self {
            Self::Back => Self::back(app),
            Self::SelectFungible(fungible) => Self::select_fungible(fungible, app),
        }
    }

    fn back(app: &'a mut App) -> Command<Message> {
        Command::none()
    }

    fn select_fungible(fungible: ResourceAddress, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Accounts(AccountsView::Account(ref mut account_view)) =
            app.appview.active_tab
        {
            if let AssetView::Fungibles(ref mut fungibles_view) = account_view.view {
                fungibles_view.selected = Some(fungible)
            } else {
                unreachable!("{}:{} Wrong State", module_path!(), line!())
            }
        } else {
            unreachable!("{}:{} Wrong State", module_path!(), line!())
        }

        Command::none()
    }
}
