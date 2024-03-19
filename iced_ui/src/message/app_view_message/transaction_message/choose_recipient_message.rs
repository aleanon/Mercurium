

use iced::Command;
use types::AccountAddress;

use crate::{message::{app_view_message::AppViewMessage, Message}, view::app_view::{transaction_view::{Recipient, View}, ActiveTab}, App};

use super::TransactionMessage;


#[derive(Debug, Clone)]
pub enum ChooseRecipientMessage {
  RecipientInput(String),
  SelectRadioButton(AccountAddress),
  Submit,
}

impl Into<Message> for ChooseRecipientMessage {
  fn into(self) -> Message {
      Message::AppView(
        AppViewMessage::TransferMessage(
          TransactionMessage::ChooseRecipientMessage(self)
        )
      )
  }
}

impl<'a> ChooseRecipientMessage {
  pub fn process(self, app: &'a mut App) -> Command<Message> {
    match self {
      Self::RecipientInput(input) => Self::update_recipient_input(input, app),
      Self::SelectRadioButton(address) => Self::set_selected_radio_button(address, app),
      Self::Submit => Self::submit_recipient(app),
    }
  }

  fn update_recipient_input(input: String, app: &'a mut App) -> Command<Message> {
    if let ActiveTab::Transfer(ref mut transaction_view)= app.appview.active_tab {
      if let View::ChooseRecipient(ref mut choose_recipient) = transaction_view.view {
        choose_recipient.recipient_input = input
        // TODO: Validate the address, if it's a valid address, push it to self.chosen_recipient
      } else {unreachable!()}
    } else {unreachable!()}

    Command::none()
  }

  fn set_selected_radio_button(address: AccountAddress, app: &'a mut App) -> Command<Message> {
    if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
      if let View::ChooseRecipient(ref mut choose_recipient) = transaction_view.view { 
        choose_recipient.chosen_account = Some(address) 
        
      } else {unreachable!()}
    }else {unreachable!()}

    Command::none()
  }

  fn submit_recipient(app: &'a mut App) -> Command<Message> {
    if let ActiveTab::Transfer(ref mut transaction_view) = app.appview.active_tab {
      if let View::ChooseRecipient(ref mut choose_recipient) = transaction_view.view {
        let recipient = Recipient::new(choose_recipient.chosen_account);
        transaction_view.recipients[choose_recipient.recipient_index] = recipient;
        transaction_view.view = View::Transaction

      } else {unreachable!()}
    } else {unreachable!()}

    Command::none()
  }
}