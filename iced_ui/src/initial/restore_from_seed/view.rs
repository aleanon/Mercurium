use iced::{
    widget::{self, column, row, text::LineHeight, text_input::Id, Button, Column, Row},
    Element, Length,
};
use types::address::Address;
use zeroize::Zeroize;

use crate::{app::AppMessage, App};

use super::{Message, RestoreFromSeed, Stage};

impl<'a> RestoreFromSeed {
    pub fn view(&'a self, _app: &'a App) -> Element<'a, AppMessage> {
        let content = match self.stage {
            Stage::EnterSeedPhrase => self.enter_seed_phrase_view(),
            Stage::EnterPassword => self.enter_password_view(),
            Stage::ChooseAccounts => self.choose_accounts_view(),
            Stage::NameAccounts => column!().into(),
            Stage::Finalizing => column!().into(),
        };

        widget::container(content)
            .center_x(660)
            .center_y(700)
            .into()
    }

    fn enter_seed_phrase_view(&self) -> Column<'a, AppMessage> {
        let input_seed = self.input_seed();

        let nav = Self::nav_row(
            Self::nav_button("Back").on_press(Message::Back.into()),
            Self::nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![input_seed, nav]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_x(iced::Alignment::Center)
            .spacing(50)
    }

    fn enter_password_view(&self) -> Column<'a, AppMessage> {
        let password_notification = Self::notification_field(self.notification);

        let password_input = Self::text_input_field("Enter Password", &self.password.as_str())
            .on_paste(|input| Message::InputPassword(input).into())
            .on_input(|input| Message::InputPassword(input).into())
            .secure(true);

        let verify_pw_input =
            Self::text_input_field("Verify Password", &self.verify_password.as_str())
                .on_submit(Message::Next.into())
                .on_paste(|input| Message::InputVerifyPassword(input).into())
                .on_input(|input| Message::InputVerifyPassword(input).into())
                .secure(true);

        let nav = Self::nav_row(
            Self::nav_button("Back").on_press(Message::Back.into()),
            Self::nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![password_notification, password_input, verify_pw_input, nav]
            .align_x(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }

    fn choose_accounts_view(&'a self) -> Column<'a, AppMessage> {
        let mut accounts = column!().height(400);

        let page = self.accounts.get(self.page_index);
        if let Some(accounts_selection) = page {
            for (i, (account, is_selected, account_summary)) in
                accounts_selection.iter().enumerate()
            {
                let account_address = widget::text(account.address.truncate());
                let account_summary = widget::text(account_summary.to_string());

                let is_selected = widget::checkbox("", *is_selected).on_toggle(move |_| {
                    Message::ToggleAccountSelection((self.page_index, i)).into()
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
            widget::button("Previous Page").on_press_maybe(if self.page_index == 0 {
                None
            } else {
                Some(Message::NewPage(self.page_index - 1).into())
            }),
            accounts.width(400),
            widget::button("Next Page").on_press(Message::NewPage(self.page_index + 1).into())
        ]
        .align_y(iced::Alignment::Center);

        let nav_buttons = Self::nav_row(
            Self::nav_button("Back").on_press(Message::Back.into()),
            Self::nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![row, nav_buttons]
            .align_x(iced::Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(50)
    }

    fn notification_field(text: &str) -> widget::Text {
        widget::text(text).size(16).width(250)
    }

    fn text_input_field(placeholder: &str, input: &str) -> widget::TextInput<'a, AppMessage> {
        widget::text_input(placeholder, input)
            .size(16)
            .width(250)
            .line_height(LineHeight::Relative(1.5))
    }

    fn seed_word_field(placeholder: &str, input: &str) -> widget::TextInput<'a, AppMessage> {
        widget::text_input(placeholder, input)
            .size(16)
            .width(100)
            .line_height(LineHeight::Relative(2.))
    }

    fn input_seed(&self) -> Column<'a, AppMessage> {
        let mut seed = widget::column![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);
        let mut row = widget::row![]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(20);

        for i in 0..24 {
            if i % 4 == 0 && i != 0 {
                seed = seed.push(row);
                row = widget::row![]
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .spacing(20);
            }
            let mut word = "";

            if let Some(s) = self.seed_phrase.reference_word(i) {
                word = s
            }

            let text_field = Self::seed_word_field(&format!("Word {}", i + 1), word)
                .id(Id::new(format!("{i}")))
                .on_input(move |input| Message::InputSeedWord((i, input)).into())
                .on_paste(move |mut string| {
                    let input = string
                        .split_ascii_whitespace()
                        .map(|s| String::from(s))
                        .collect::<Vec<String>>();
                    string.zeroize();
                    Message::PasteSeedPhrase((i, input)).into()
                });

            row = row.push(text_field);
        }
        seed.push(row)
    }

    fn nav_button(text: &'a str) -> Button<'a, AppMessage> {
        Button::new(
            widget::text(text)
                .size(16)
                .width(50)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
    }

    pub fn nav_row(
        back: Button<'a, AppMessage>,
        next: Button<'a, AppMessage>,
    ) -> Row<'a, AppMessage> {
        let space = widget::Space::with_width(Length::Fill);
        widget::row![back, space, next]
            .width(Length::Fill)
            .align_y(iced::Alignment::Start)
    }
}
