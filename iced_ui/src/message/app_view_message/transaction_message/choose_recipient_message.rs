use iced::Command;
use types::AccountAddress;

use crate::{
    message::{app_view_message::AppViewMessage, Message},
    view::app_view::{
        transaction_view::{
            TransactionView, View,
        },
    },
};

use super::TransactionMessage;

#[derive(Debug, Clone)]
pub enum ChooseRecipientMessage {
    RecipientInput(String),
    SelectRadioButton(AccountAddress),
    Submit,
}

impl Into<Message> for ChooseRecipientMessage {
    fn into(self) -> Message {
        Message::AppView(AppViewMessage::TransferMessage(
            TransactionMessage::ChooseRecipientMessage(self),
        ))
    }
}

impl<'a> ChooseRecipientMessage {
    pub fn process(self, parent: &'a mut TransactionView) -> Command<Message> {
        let view = &mut parent.view;

        if let View::ChooseRecipient(ref mut choose_recipient) = view {
            let command = Command::none();

            match self {
                Self::RecipientInput(input) => choose_recipient.recipient_input = input,
                Self::SelectRadioButton(address) => choose_recipient.chosen_account = Some(address),
                Self::Submit => {
                    parent.recipients[choose_recipient.recipient_index].address =
                        choose_recipient.chosen_account.take();
                    parent.view = View::Transaction;
                }
            }

            command
        } else {
            unreachable!("{}:{} Invalid state", module_path!(), line!())
        }
    }

    // fn update_recipient_input(input: String, app: &'a mut App) -> Command<Message> {
    //   if let ActiveTab::Transfer(ref mut transaction_view)= app.appview.active_tab {
    //     if let View::ChooseRecipient(ref mut choose_recipient) = transaction_view.view {
    //       choose_recipient.recipient_input = input
    //       // TODO: Validate the address, if it's a valid address, push it to self.chosen_recipient
    //     } else {unreachable!()}
    //   } else {unreachable!()}

    //   Command::none()
    // }

    // fn set_selected_radio_button(address: AccountAddress, app: &'a mut App) -> Command<Message> {
    //   if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
    //     if let View::ChooseRecipient(ref mut choose_recipient) = transaction_view.view {
    //       choose_recipient.chosen_account = Some(address)

    //     } else {unreachable!()}
    //   }else {unreachable!()}

    //   Command::none()
    // }

    // fn submit_recipient(transaction_view: &'a mut TransactionView, chosen_account: Option<AccountAddress>, recipient_index: usize) -> Command<Message> {
    //     let recipient = Recipient::new(chosen_account);
    //     transaction_view.recipients[recipient_index] = recipient;
    //     transaction_view.view = View::Transaction;

    //   Command::none()
    // }
}
