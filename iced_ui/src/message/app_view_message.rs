pub mod accounts_message;
pub mod overlay_message;
pub mod transaction_message;

use std::{collections::HashMap, str::FromStr};

use iced::Command;
use types::{Account, Decimal, RadixDecimal, ResourceAddress};

use crate::{
    app::App,
    view::app_view::{
        accounts_view::AccountsView, overlay::Overlay, transaction_view::TransactionView,
        ActiveTab, TabId,
    },
};

use self::{
    accounts_message::AccountsViewMessage, overlay_message::OverlayMessage,
    transaction_message::TransactionMessage,
};

use super::Message;

#[derive(Debug, Clone)]
pub enum AppViewMessage {
    SelectTab(TabId),
    AccountsOverview,
    AccountsViewMessage(AccountsViewMessage),
    NewTransaction(Option<Account>),
    TransferMessage(TransactionMessage),
    SpawnOverlay(Overlay),
    CloseOverlay,
    OverlayMessage(OverlayMessage),
}

impl Into<Message> for AppViewMessage {
    fn into(self) -> Message {
        Message::AppView(self)
    }
}

impl<'a> AppViewMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();

        match self {
            Self::SelectTab(tab_id) => Self::select_tab(tab_id, app),
            Self::AccountsOverview => {
                app.appview.active_tab = ActiveTab::Accounts(AccountsView::new())
            }
            Self::NewTransaction(from_account) => Self::new_transaction(from_account, app),
            Self::AccountsViewMessage(accounts_message) => command = accounts_message.process(app),
            Self::TransferMessage(transfer_message) => command = transfer_message.process(app),
            Self::SpawnOverlay(overlay) => app.appview.overlay = Some(overlay),
            Self::CloseOverlay => app.appview.overlay = None,
            Self::OverlayMessage(overlay_message) => command = overlay_message.update(app),
            // Self::CenterPanelMessage(center_panel_message) => center_panel_message.process(app),
            // Self::MenuMessage(menu_message) => menu_message.process(app),
        }

        command
    }

    fn select_tab(tab_id: TabId, app: &'a mut App) {
        match tab_id {
            TabId::Accounts => app.appview.active_tab = ActiveTab::Accounts(AccountsView::new()),
            TabId::Transfer => {
                app.appview.active_tab = ActiveTab::Transfer(TransactionView::new(None, None))
            }
        }
    }

    fn new_transaction(from_account: Option<Account>, app: &'a mut App) {
        match from_account {
            Some(ref account) => {
                let asset_amounts =
                    app.app_data
                        .fungibles
                        .get(&account.address)
                        .and_then(|fungibles| {
                            Some(
                                fungibles
                                    .into_iter()
                                    .filter_map(|fungible| {
                                        Some((
                                            fungible.resource_address.clone(),
                                            RadixDecimal::from_str(&fungible.amount).ok()?.into(),
                                        ))
                                    })
                                    .collect::<HashMap<ResourceAddress, Decimal>>(),
                            )
                        });

                app.appview.active_tab =
                    ActiveTab::Transfer(TransactionView::new(from_account, asset_amounts));
            }
            None => {
                app.appview.active_tab = ActiveTab::Transfer(TransactionView::new(None, None));
            }
        }
    }
}
