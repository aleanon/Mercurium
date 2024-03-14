use iced::{Element, widget};

use crate::{app::App, message::{setup_message::SetupMessage, Message}};

use super::Setup; 


#[derive(Debug)]
pub enum RestoreWallet {
    FromCloud,
    FromFile,
}


impl<'a> RestoreWallet {
    pub fn view(&self, _app: &'a App) -> Element<'a, Message> {
        let from_backup = Setup::creation_button("From Cloud")
                    .on_press(SetupMessage::Restore.into());

        let from_seed = Setup::creation_button("From File")
            .on_press(SetupMessage::Restore.into());

        
        widget::column![from_backup, from_seed]
            .into()
    }
}