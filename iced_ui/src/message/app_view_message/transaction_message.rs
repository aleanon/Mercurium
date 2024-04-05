pub mod choose_recipient_message;
pub mod add_assets_message;

use iced::Command;

use crate::{app::{self, App}, message::Message, view::app_view::{transaction_view::{self, add_assets::AddAssets, choose_recipient::ChooseRecipient, Recipient, View}, ActiveTab}};
use types::{Account, AccountAddress, Decimal, ResourceAddress};

use self::{add_assets_message::AddAssetsMessage, choose_recipient_message::ChooseRecipientMessage};

use super::{AppViewMessage, TransactionView};

#[derive(Debug, Clone)]
pub enum TransactionMessage {
    SelectAccount(Account),
    UpdateMessage(String),
    RemoveRecipient(usize),
    UpdateResourceAmount(usize , usize, String),
    SelectRecipient(usize),
    AddRecipient,
    ChooseRecipientMessage(ChooseRecipientMessage),
    ///Pass the index of the account to add assets for
    AddAssets{recipient_index: usize, from_account: AccountAddress},
    AddAssetsMessage(AddAssetsMessage),
}

impl Into<Message> for TransactionMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::TransferMessage(self))
    }
}

impl<'a> TransactionMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let active_tab = &mut app.appview.active_tab;

        if let ActiveTab::Transfer(transaction_view) = active_tab {
            let mut command = Command::none();

            match self {
                Self::SelectAccount(account) => transaction_view.from_account = Some(account),
                Self::UpdateMessage(message) => transaction_view.message = message,
                Self::RemoveRecipient(recipient_index) => Self::remove_recipient(recipient_index, transaction_view),
                Self::UpdateResourceAmount(account_index, resource, amount) => Self::update_resource_amount(account_index, resource, amount, transaction_view),
                Self::SelectRecipient(recipient_index) => transaction_view.view = View::ChooseRecipient(ChooseRecipient::new(recipient_index)),
                Self::AddRecipient => transaction_view.recipients.push(Recipient::new(None)),
                Self::AddAssets { recipient_index, from_account } => transaction_view.view = View::ChooseResource(AddAssets::new(from_account, recipient_index)),
                Self::AddAssetsMessage(add_assets_message) => command = add_assets_message.process(transaction_view),
                Self::ChooseRecipientMessage(choose_recipient_message) => command = choose_recipient_message.process(transaction_view),
            }

            command

        } else {
            unreachable!("{}:{} Invalid state", module_path!(), line!())
        }
    }

    // fn select_account(account: Account, app: &'a mut App) -> Command<Message> {
    //     if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
    //         transaction_view.from_account = Some(account);
    //     } else {
    //         unreachable!("{}:{} Invalid state", module_path!(), line!())
    //     }

    //     Command::none()
    // }

    // fn update_message(message: String, app: &'a mut App) -> Command<Message> {
    //     if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
    //         transaction_view.message = message
    //     } else {
    //         unreachable!("{}:{} Invalid state", module_path!(), line!())
    //     }

    //     Command::none()
    // }

    fn remove_recipient(index: usize, transaction_view: &'a mut TransactionView) {
        if transaction_view.recipients.len() == 1 {
            transaction_view.recipients[index].address = None;
            transaction_view.recipients[index].resources.clear();
            
        } else if transaction_view.recipients.len() > index {
                transaction_view.recipients.remove(index);
        }
    }

    fn update_resource_amount(account_index: usize , resource_index: usize, new_amount: String, transaction_view: &'a mut TransactionView) {
        //checks that the input value is a valid Decimal type for the Radix network
        if let Ok(_) = types::RadixDecimal::try_from(new_amount.as_bytes()) {
            transaction_view.recipients[account_index].resources[resource_index].2 =new_amount;
        }
    }

    // fn open_recipient_selection(recipient_index: usize, transaction_view: &'a mut TransactionView) -> Command<Message> {
    //     transaction_view.view = View::ChooseRecipient(ChooseRecipient::new(recipient_index));

    //     Command::none()
    // }

    // fn add_recipient(transaction_view: &'a mut TransactionView) -> Command<Message> {
    //     transaction_view.recipients.push(Recipient::new(None));

    //     Command::none()
    // }


    // fn open_asset_selection(recipient_index: usize, from_account: AccountAddress, transaction_view: &'a mut TransactionView) -> Command<Message> {
    //     transaction_view.view = View::ChooseResource(AddAssets::new(from_account, recipient_index));

    //     Command::none()
    // }

}
