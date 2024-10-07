use iced::{
    widget::{self, column, row, Column},
    Length,
};
use types::address::Address;

use crate::{
    app::AppMessage,
    initial::common::{nav_button, nav_row},
};

use super::{Message, RestoreFromSeed};

impl<'a> RestoreFromSeed {
    pub fn choose_accounts_view(&'a self) -> Column<'a, AppMessage> {
        let mut accounts = column!().height(400);

        let page = self
            .accounts_data
            .accounts
            .get(self.accounts_data.page_index);
        if let Some(accounts_selection) = page {
            for (i, (account, is_selected, account_summary)) in
                accounts_selection.iter().enumerate()
            {
                let account_address = widget::text(account.address.truncate());
                let account_summary = widget::text(account_summary.to_string());

                let is_selected = widget::checkbox("", *is_selected).on_toggle(move |_| {
                    Message::ToggleAccountSelection((self.accounts_data.page_index, i)).into()
                });

                accounts = accounts.push(
                    row![
                        account_address.width(Length::FillPortion(10)),
                        widget::Space::new(Length::Fill, 1),
                        account_summary.width(Length::FillPortion(10)),
                        widget::Space::new(Length::FillPortion(2), 1),
                        is_selected.width(Length::FillPortion(2))
                    ]
                    .width(Length::Fill),
                )
            }
        }

        let row = row![
            widget::button("Previous Page").on_press_maybe(if self.accounts_data.page_index == 0 {
                None
            } else {
                Some(Message::NewPage(self.accounts_data.page_index - 1).into())
            }),
            accounts.width(400),
            widget::button("Next Page")
                .on_press(Message::NewPage(self.accounts_data.page_index + 1).into())
        ]
        .align_y(iced::Alignment::Center);

        let nav_buttons = nav_row(
            nav_button("Back").on_press(Message::Back.into()),
            nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![row, nav_buttons]
            .align_x(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }
}
