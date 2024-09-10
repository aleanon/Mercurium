use iced::{widget, Element};

use crate::{app::App, app::AppMessage};

use super::setup::{self, Setup};

#[derive(Debug, Clone)]
pub enum Message {
    FromBackup,
    FromFile,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(setup::Message::RestoreWalletMessage(self))
    }
}

#[derive(Debug)]
pub enum RestoreWallet {
    FromBackup,
    FromSeed,
}

impl<'a> RestoreWallet {
    pub fn view(&self, _app: &'a App) -> Element<'a, AppMessage> {
        let from_backup =
            Setup::creation_button("From Cloud").on_press(setup::Message::FromBackup.into());

        let from_seed =
            Setup::creation_button("From File").on_press(setup::Message::FromBackup.into());

        widget::column![from_backup, from_seed].into()
    }
}
