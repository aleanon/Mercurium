use iced::{
    theme,
    widget::{self, button, column, container::StyleSheet, row, text, text_input},
    Element, Length,
};
use ravault_iced_theme::styles;
use types::crypto::Password;

use crate::{
    message::{app_view_message::overlay_message::add_account_message::AddAccountMessage, Message},
    App,
};
#[derive(Debug, Clone)]
pub enum View {
    InputAccountName,
    InputPassword,
}

#[derive(Debug, Clone)]
pub struct AddAccountView {
    pub notification: String,
    pub account_name: String,
    pub password: Password,
    pub view: View,
}

impl AddAccountView {
    pub fn new() -> Self {
        Self {
            notification: String::new(),
            account_name: String::new(),
            password: Password::new(),
            view: View::InputAccountName,
        }
    }
}

impl<'a> AddAccountView {
    pub fn view(&self, app: &'a App) -> Element<'a, Message> {
        let content = match self.view {
            View::InputAccountName => self.input_account_name(app),
            View::InputPassword => self.input_password(app),
        };
        let notification = text(&self.notification);

        let column = column![notification, content];

        widget::container(column)
            .width(400)
            .height(400)
            .padding(10)
            .center_x()
            .center_y()
            .style(styles::container::OverlayInner::style)
            .into()
    }

    fn input_account_name(&self, app: &'a App) -> Element<'a, Message> {
        let account_name_input = {
            let label = text("Account name");
            let account_name_input = text_input("Enter account name", &self.account_name)
                .style(theme::TextInput::Custom(Box::new(
                    styles::text_input::GeneralInput,
                )))
                .on_input(|input| AddAccountMessage::InputAccountName(input).into());

            column!(label, account_name_input).spacing(2)
        };

        let next_button = button("Next").on_press(AddAccountMessage::Next.into());

        column![account_name_input, next_button].spacing(20).into()
    }

    fn input_password(&self, app: &'a App) -> Element<'a, Message> {
        let password_input = {
            let label = text("Password");
            let password_input = text_input("Enter password", &self.password.as_str())
                .style(theme::TextInput::Custom(Box::new(
                    styles::text_input::GeneralInput,
                )))
                .on_input(|input| AddAccountMessage::InputPassword(input).into());

            column![label, password_input].spacing(2)
        };

        let submit_button = button("Submit").on_press(AddAccountMessage::Submit.into());

        column![password_input, submit_button].spacing(20).into()
    }
}
