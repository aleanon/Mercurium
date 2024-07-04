use std::collections::{BTreeMap, BTreeSet};

use iced::{
    theme,
    widget::{self, button, column, container, row, text},
    Command, Element, Length, Padding,
};
use ravault_iced_theme::styles;
use types::{Account, AccountAddress};

use crate::{app::AppData, app::AppMessage, unlocked::app_view};

use super::transaction_view::{self, Recipient};

#[derive(Debug, Clone)]
pub enum Message {
    RecipientInput(String),
    SelectRadioButton(AccountAddress),
    Submit,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::TransactionMessage(
            transaction_view::Message::ChooseRecipientMessage(self),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct ChooseRecipient {
    pub recipient_index: usize,
    pub recipient_input: String,
    pub selected_radio: Option<usize>,
    pub chosen_account: Option<AccountAddress>,
}

impl ChooseRecipient {
    pub fn new(recipient_index: usize) -> Self {
        Self {
            recipient_index,
            recipient_input: String::new(),
            selected_radio: None,
            chosen_account: None,
        }
    }
}

impl<'a> ChooseRecipient {
    pub fn update(
        &mut self,
        message: Message,
        recipients: &'a mut Vec<Recipient>,
    ) -> Command<AppMessage> {
        match message {
            Message::RecipientInput(input) => self.recipient_input = input,
            Message::SelectRadioButton(address) => self.chosen_account = Some(address),
            Message::Submit => return self.submit(recipients),
        }

        Command::none()
    }

    fn submit(&mut self, recipients: &'a mut Vec<Recipient>) -> Command<AppMessage> {
        recipients[self.recipient_index].address = self.chosen_account.take();
        Command::perform(async {}, |_| transaction_view::Message::OverView.into())
    }

    pub fn view(&self, appdata: &'a AppData) -> Element<'a, AppMessage> {
        let accounts = appdata
            .db
            .get_accounts()
            .unwrap_or(BTreeMap::new())
            .into_iter()
            .map(|(_, account)| account)
            .collect::<BTreeSet<Account>>();

        let header = widget::text("Add recipient")
            .line_height(2.)
            .size(20)
            .width(Length::Fill)
            .height(Length::Shrink)
            .horizontal_alignment(iced::alignment::Horizontal::Center);

        let space = widget::Space::new(Length::Fill, 50);

        let text_input = widget::text_input("Enter recipient address", &self.recipient_input)
            .width(Length::Fill)
            .line_height(2.)
            .on_input(|value| Message::RecipientInput(value).into())
            .on_paste(|value| Message::RecipientInput(value).into());

        let space2 = widget::Space::new(Length::Fill, 20);

        let mut buttons = column!();

        for (i, account) in accounts.into_iter().enumerate() {
            let selected = self.chosen_account.as_ref().and_then(|address| {
                if address == &account.address {
                    Some(i)
                } else {
                    None
                }
            });

            let account_name = text(account.name)
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
                .width(Length::Shrink);

            let button_row = row![name_and_address, space, radio]
                .align_items(iced::Alignment::Center)
                .width(Length::Fill)
                .height(Length::Shrink);

            let message: Option<AppMessage> = if self.recipient_input.is_empty() {
                Some(Message::SelectRadioButton(account.address).into())
            } else {
                None
            };
            let button = button(button_row)
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

        let buttons = widget::scrollable(buttons.padding(Padding {
            right: 15.,
            ..Padding::ZERO
        }))
        .style(theme::Scrollable::custom(styles::scrollable::Scrollable))
        .width(Length::Fill)
        .height(Length::Shrink);

        let col = column![header, space, text_input, space2, buttons].width(500);

        let main_content = container(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x();

        let submit = button("Submit").on_press_maybe({
            self.chosen_account
                .as_ref()
                .and_then(|_| Some(Message::Submit.into()))
        });
        let submit = container(submit).width(Length::Fill).padding(10).center_x();

        column!(main_content, submit)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}
