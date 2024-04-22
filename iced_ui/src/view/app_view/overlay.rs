pub mod add_account_view;
pub mod receive;

use iced::{
    widget::{self, container::StyleSheet},
    Element, Length,
};
use ravault_iced_theme::styles;

use crate::{message::Message, App};

use self::{add_account_view::AddAccountView, receive::Receive};

#[derive(Debug, Clone)]
pub enum Overlay {
    AddAccount(AddAccountView),
    Receive(Receive),
}

impl<'a> Overlay {
    pub fn view(&'a self, app: &'a App) -> Element<'a, Message> {
        match self {
            Self::AddAccount(add_account_view) => add_account_view.view(app),
            Self::Receive(receive) => receive.view(app),
        }
    }
}
