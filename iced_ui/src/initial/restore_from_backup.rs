use deps::*;

use iced::{widget::column, Element};

use crate::{app::App, app::AppMessage};

use super::setup;

#[derive(Debug, Clone)]
pub enum Message {
    FromBackup,
    FromFile,
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::Setup(setup::Message::RestoreFromBackupMessage(self))
    }
}

#[derive(Debug)]
pub enum RestoreFromBackup {
    FromBackup,
    FromSeed,
}

impl<'a> RestoreFromBackup {
    pub fn view(&self, _app: &'a App) -> Element<'_, Message> {
        column!().into()
        // let from_backup =
        //     Setup::creation_button("From Cloud").on_press(setup::Message::FromBackup);

        // let from_seed =
        //     Setup::creation_button("From File").on_press(setup::Message::FromBackup);

        // column![from_backup, from_seed].into()
    }
}
