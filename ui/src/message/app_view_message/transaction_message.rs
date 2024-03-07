use iced::Command;

use crate::{app::App, message::Message, view::app_view::{transaction_view::View, ActiveTab}};
use types::{Account, AccountAddress, Decimal, ResourceAddress};

use super::{AppViewMessage, TransactionView};

#[derive(Debug, Clone)]
pub enum TransferMessage {
    ChooseAccount,
    FromAccount(Account),
    UpdateMessage(String),
    RemoveRecipient(AccountAddress),
    UpdateResourceAmount((AccountAddress ,ResourceAddress, String)),
}

impl Into<Message> for TransferMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::TransferMessage(self))
    }
}

impl<'a> TransferMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self {
            Self::ChooseAccount => Self::choose_account(app),
            Self::FromAccount(account) => Self::from_account(account, app),
            Self::UpdateMessage(message) => Self::update_message(message, app),
            Self::RemoveRecipient(recipient) => Self::remove_recipient(recipient, app),
            Self::UpdateResourceAmount((account, resource, amount)) => Self::update_resource_amount(account, resource, amount, app)
        }
    }

    fn choose_account(app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            transaction_view.view = View::ChooseFromAccount
        } else {
            unreachable!("{}:{} Invalid state", module_path!(), line!())
        }

        Command::none()
    }

    fn from_account(account: Account, app: &'a mut App) -> Command<Message> {
        app.appview.active_tab = ActiveTab::Transfer(TransactionView::from_account(account));

        Command::none()
    }

    fn update_message(message: String, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            transaction_view.message = message
        } else {
            unreachable!("{}:{} Invalid state", module_path!(), line!())
        }

        Command::none()
    }

    fn remove_recipient(recipient: AccountAddress, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            for i in 0..transaction_view.recipients.len() {
                if transaction_view.recipients[i].address == recipient {
                    transaction_view.recipients.remove(i);
                    break;
                } 
            }
        } else {
            unreachable!("{},{} Invalid state", module_path!(), line!())
        }
        Command::none()
    }

    fn update_resource_amount(account: AccountAddress, resource: ResourceAddress, new_amount: String, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            for recipient in transaction_view.recipients.iter_mut() {
                if recipient.address == account {
                    for (_, address, amount) in recipient.resources.iter_mut() {
                        if *address == resource {
                            if let Ok(_) = types::RadixDecimal::try_from(new_amount.as_bytes()) {
                                *amount = new_amount;
                            }
                            break;
                        }
                    }
                    break;
                }
            }
        } else {
            unreachable!("{},{} Invalid state", module_path!(), line!())
        }
        Command::none()
    }
}
