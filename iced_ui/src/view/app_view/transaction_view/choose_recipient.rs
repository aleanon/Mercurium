use std::collections::{BTreeMap, BTreeSet};

use iced::{
    theme,
    widget::{self, button, column, container, row, text},
    Element, Length, Padding,
};
use ravault_iced_theme::styles;
use types::{Account, AccountAddress};

use crate::{
    message::{
        app_view_message::transaction_message::choose_recipient_message::ChooseRecipientMessage,
        Message,
    },
    App,
};

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
    pub fn view(&self, app: &'a App) -> Element<'a, Message> {
        let accounts = app
            .app_data
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
            .on_input(|value| ChooseRecipientMessage::RecipientInput(value).into())
            .on_paste(|value| ChooseRecipientMessage::RecipientInput(value).into());

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
            let radio =
                widget::radio(String::new(), i, selected, |_| Message::None).width(Length::Shrink);

            let button_row = row![name_and_address, space, radio]
                .align_items(iced::Alignment::Center)
                .width(Length::Fill)
                .height(Length::Shrink);

            let message: Option<Message> = if self.recipient_input.is_empty() {
                Some(ChooseRecipientMessage::SelectRadioButton(account.address).into())
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
                .and_then(|_| Some(ChooseRecipientMessage::Submit.into()))
        });
        let submit = container(submit).width(Length::Fill).padding(10).center_x();

        column!(main_content, submit)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}
