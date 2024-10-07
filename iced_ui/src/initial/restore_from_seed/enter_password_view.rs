use iced::{
    widget::{self, Column},
    Length,
};

use crate::{
    app::AppMessage,
    common_elements,
    initial::common::{nav_button, nav_row, notification_field, text_input_field},
};

use super::{Message, RestoreFromSeed};

impl<'a> RestoreFromSeed {
    pub fn enter_password_view(&self) -> Column<'a, AppMessage> {
        let header = common_elements::header_one("Create password");

        let password_notification = notification_field(self.notification);

        let password_input = text_input_field("Enter Password", &self.inputs.password.as_str())
            .on_paste(|input| Message::InputPassword(input).into())
            .on_input(|input| Message::InputPassword(input).into())
            .secure(true);

        let verify_pw_input =
            text_input_field("Verify Password", &self.inputs.verify_password.as_str())
                .on_submit(Message::Next.into())
                .on_paste(|input| Message::InputVerifyPassword(input).into())
                .on_input(|input| Message::InputVerifyPassword(input).into())
                .secure(true);

        let nav = nav_row(
            nav_button("Back").on_press(Message::Back.into()),
            nav_button("Next").on_press(Message::Next.into()),
        );

        widget::column![
            header,
            password_notification,
            password_input,
            verify_pw_input,
            nav
        ]
        .align_x(iced::Alignment::Center)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .spacing(50)
    }
}
