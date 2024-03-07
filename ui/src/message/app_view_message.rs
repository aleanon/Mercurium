pub mod transaction_message;
pub mod accounts_message;

use iced::Command;

use crate::{app::App, view::app_view::{accounts_view::AccountsView, transaction_view::TransactionView, ActiveTab, TabId}}; 

use self::{accounts_message::AccountsViewMessage, transaction_message::TransferMessage};

use super::Message;


#[derive(Debug, Clone)]
pub enum AppViewMessage {
    SelectTab(TabId),
    AccountsViewMessage(AccountsViewMessage),
    TransferMessage(TransferMessage),
}

impl Into<Message> for AppViewMessage {
    fn into(self) -> Message {
        Message::AppView(self)
    }
}

impl<'a> AppViewMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self {
            Self::SelectTab(tab_id) => Self::select_tab(tab_id, app),
            Self::AccountsViewMessage(accounts_message) => accounts_message.process(app),
            Self::TransferMessage(transfer_message) => transfer_message.process(app),
            // Self::CenterPanelMessage(center_panel_message) => center_panel_message.process(app),
            // Self::MenuMessage(menu_message) => menu_message.process(app),
        }
    }

    fn select_tab(tab_id: TabId, app: &'a mut App) -> Command<Message> {
        match tab_id {
            TabId::Accounts => app.appview.active_tab = ActiveTab::Accounts(AccountsView::new()),
            TabId::Transfer => app.appview.active_tab = ActiveTab::Transfer(TransactionView::new()),
        }

        Command::none()
    }
}
