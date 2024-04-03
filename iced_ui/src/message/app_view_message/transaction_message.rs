pub mod choose_recipient_message;

use iced::Command;

use crate::{app::App, message::Message, view::app_view::{transaction_view::{choose_recipient::ChooseRecipient, Recipient, View}, ActiveTab}};
use types::{Account, AccountAddress, Decimal, ResourceAddress};

use self::choose_recipient_message::ChooseRecipientMessage;

use super::{AppViewMessage, TransactionView};

#[derive(Debug, Clone)]
pub enum TransactionMessage {
    FromAccount(Account),
    SelectAccount(Account),
    UpdateMessage(String),
    RemoveRecipient(usize),
    UpdateResourceAmount(usize , usize, String),
    SelectRecipient(usize),
    SelectRadioButton(usize),
    AddRecipient,
    ChooseRecipientMessage(ChooseRecipientMessage),
}

impl Into<Message> for TransactionMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::TransferMessage(self))
    }
}

impl<'a> TransactionMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self {
            Self::FromAccount(account) => Self::from_account(account, app),
            Self::SelectAccount(account) => Self::select_account(account, app),
            Self::UpdateMessage(message) => Self::update_message(message, app),
            Self::RemoveRecipient(recipient_index) => Self::remove_recipient(recipient_index, app),
            Self::UpdateResourceAmount(account_index, resource, amount) => Self::update_resource_amount(account_index, resource, amount, app),
            Self::SelectRecipient(recipient_index) => Self::select_recipient(recipient_index, app),
            Self::SelectRadioButton(id) => Self::select_radio_button(id, app),
            Self::AddRecipient => Self::add_recipient(app),
            Self::ChooseRecipientMessage(choose_recipient_message) => choose_recipient_message.process(app),
        }
    }


    fn from_account(account: Account, app: &'a mut App) -> Command<Message> {
        app.appview.active_tab = ActiveTab::Transfer(TransactionView::from_account(account));

        Command::none()
    }

    fn select_account(account: Account, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            transaction_view.from_account = Some(account);
        } else {
            unreachable!("{}:{} Invalid state", module_path!(), line!())
        }

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

    fn remove_recipient(index: usize, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            if transaction_view.recipients.len() == 1 {
                transaction_view.recipients[index].address = None;
                transaction_view.recipients[index].resources.clear();
                
            } else if transaction_view.recipients.len() > index {
                    transaction_view.recipients.remove(index);
            }
        } else {
            unreachable!("{},{} Invalid state", module_path!(), line!())
        }

        Command::none()
    }

    fn update_resource_amount(account_index: usize , resource_index: usize, new_amount: String, app: &'a mut App) -> Command<Message> {
        //checks that the input value is a valid Decimal type for the Radix network
        if let Err(_) = types::RadixDecimal::try_from(new_amount.as_bytes()) {
            return Command::none()
        }

        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            transaction_view.recipients[account_index].resources[resource_index].2 =new_amount;

        } else {
            unreachable!("{},{} Invalid state", module_path!(), line!())
        }

        Command::none()
    }

    fn select_recipient(recipient_index: usize, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            transaction_view.view = View::ChooseRecipient(ChooseRecipient::new(recipient_index))
        } else {
            unreachable!("{}, {} Invalid state", module_path!(), line!())
        }

        Command::none()
    }

    fn add_recipient(app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            transaction_view.recipients.push(Recipient::new(None))
        } else {
            unreachable!()
        }

        Command::none()
    }

    fn select_radio_button(id: usize, app: &'a mut App) -> Command<Message> {
        if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
            if let View::ChooseRecipient(ref mut choose_recipient) = transaction_view.view {
                choose_recipient.selected_radio = Some(id)
            }
        } else {
            unreachable!("{}, {} Invalid state", module_path!(), line!())
        }

        Command::none()
    }

}
