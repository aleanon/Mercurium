use deps::{iced::alignment::Horizontal, *};

use std::str::FromStr;

use iced::{
    Element, Length, Padding, Task,
    widget::{self, button, column, container, row, text},
};
use types::address::{AccountAddress, Address};
use wallet::{Unlocked, Wallet};

use crate::{app::AppMessage, styles, unlocked::app_view};

use super::create_transaction::{self, Recipient};

#[derive(Debug, Clone)]
pub enum Message {
    RecipientInput(String),
    SelectRadioButton(AccountAddress),
    Submit,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::TransactionMessage(
            create_transaction::Message::ChooseRecipientMessage(self),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct AddRecipient {
    pub from_account: Option<AccountAddress>,
    pub recipient_index: usize,
    pub recipient_input: String,
    pub selected_radio: Option<usize>,
    pub chosen_account: Option<AccountAddress>,
}

impl AddRecipient {
    pub fn new(recipient_index: usize, from_account: Option<AccountAddress>) -> Self {
        Self {
            from_account,
            recipient_index,
            recipient_input: String::new(),
            selected_radio: None,
            chosen_account: None,
        }
    }
}

impl<'a> AddRecipient {
    pub fn update(
        &mut self,
        message: Message,
        recipients: &'a mut Vec<Recipient>,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        match message {
            Message::RecipientInput(input) => self.recipient_input(input, wallet),
            Message::SelectRadioButton(address) => self.chosen_account = Some(address),
            Message::Submit => return self.submit(recipients),
        }

        Task::none()
    }

    fn recipient_input(&mut self, input: String, wallet: &'a mut Wallet<Unlocked>) {
        if let Ok(account_address) = AccountAddress::from_str(input.as_str()) {
            if let Some(address) = &self.from_account {
                if &account_address != address {
                    self.chosen_account = Some(account_address)
                }
            } else {
                self.chosen_account = Some(account_address);
            }
            self.recipient_input = input;
        } else {
            self.chosen_account = None;
            self.recipient_input = input;
        }
    }

    fn submit(&mut self, recipients: &'a mut Vec<Recipient>) -> Task<AppMessage> {
        recipients[self.recipient_index].address = self.chosen_account.take();
        Task::perform(async {}, |_| create_transaction::Message::OverView.into())
    }

    pub fn view(&self, wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        let header = widget::text("Add recipient")
            .line_height(2.)
            .size(20)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(iced::alignment::Horizontal::Center);

        let space = widget::Space::new(Length::Fill, 50);

        let text_input = widget::text_input("Enter recipient address", &self.recipient_input)
            .width(Length::Fill)
            .line_height(2.)
            .style(styles::text_input::base_layer_1_rounded)
            .on_input(|value| Message::RecipientInput(value).into())
            .on_paste(|value| Message::RecipientInput(value).into());

        let text_input = container(text_input).padding(Padding {
            left: 15.,
            right: 15.,
            ..Padding::ZERO
        });

        let space2 = widget::Space::new(Length::Fill, 20);

        let mut buttons = column!().align_x(Horizontal::Center).width(Length::Fill);

        for (i, (_, account)) in wallet
            .accounts()
            .iter()
            .filter(|(account_address, _)| Some(*account_address) != self.from_account.as_ref())
            .enumerate()
        {
            let selected = self.chosen_account.as_ref().and_then(|address| {
                if address == &account.address {
                    Some(i)
                } else {
                    None
                }
            });

            let account_name = text(account.name.as_str())
                .line_height(2.)
                .size(12)
                .width(Length::Shrink);
            let account_address = text(account.address.truncate_long())
                .line_height(1.5)
                .size(10)
                .width(Length::Shrink);
            let name_and_address = column![account_name, account_address].spacing(2);
            let space = widget::Space::new(Length::Fill, 1);
            let radio = widget::radio(String::new(), i, selected, |_| AppMessage::None)
                .width(Length::Shrink)
                .size(20);

            let button_row = row![name_and_address, space, radio]
                .align_y(iced::Alignment::Center)
                .width(Length::Fill)
                .height(Length::Shrink);

            let message: Option<AppMessage> = if self.recipient_input.is_empty() {
                Some(Message::SelectRadioButton(account.address.clone()).into())
            } else {
                None
            };

            let button = button(button_row)
                .style(styles::button::base_layer_2_rounded_with_shadow)
                .padding(10)
                .width(Length::Fill)
                .height(Length::Shrink)
                .on_press_maybe(message);

            buttons = buttons.push(button)
        }

        buttons = buttons
            .spacing(10)
            .width(Length::Fill)
            .height(Length::Shrink);

        let buttons = widget::scrollable(buttons.padding(15))
            .style(styles::scrollable::vertical_scrollable_secondary)
            .width(Length::Fill)
            .height(Length::Shrink);

        let col = column![header, space, text_input, space2, buttons].width(500);

        let main_content = container(col).height(Length::Fill).center_x(Length::Fill);

        let submit = button("Submit").on_press_maybe({
            self.chosen_account
                .as_ref()
                .and_then(|_| Some(Message::Submit.into()))
        });
        let submit = container(submit).padding(10).center_x(Length::Fill);

        column!(main_content, submit)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}
