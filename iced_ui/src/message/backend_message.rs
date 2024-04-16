use std::collections::HashMap;

use debug_print::debug_println;

use iced::widget::image::Handle;
use iced::Command;
use iced::futures::SinkExt;


use types::{Action, EntityAccount, ResourceAddress, Update};

use crate::{message::Message, App};

// Uses a newtype wrapper because the Update type is used in the backend crate and has to be defined in the types crate
#[derive(Debug, Clone)]
pub struct BackendMessage(pub(crate)Update);


impl<'a> BackendMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();
        match self.0 {
            Update::Sender(action_tx) => app.app_data.backend_sender = action_tx,
            Update::Icons(icons) => Self::store_icons_in_cache(icons, app), 
            Update::Accounts(accounts) => Self::save_updated_data(accounts, app),
            Update::DatabaseLoaded => command = Self::send_update_all_request(app),
            _ => {}
        }
        command
    }


    fn store_icons_in_cache(icons: HashMap<ResourceAddress, Handle>, app: &'a mut App) {
       debug_println!(
            "{}:{} Received {}icons:",
            module_path!(),
            line!(),
            icons.len()
        );
        app.appview.resource_icons = icons; 
    }

    #[cfg(not(feature = "noupdate"))]
    fn send_update_all_request(app: &'a mut App) -> Command<Message> {
        let mut channel = app.app_data.backend_sender.clone();
        Command::perform(
            async move { channel.send(Action::UpdateAll).await },
            |_| Message::None,
        )
    }

    #[cfg(feature = "noupdate")]
    fn send_update_all_request(app:&'a mut App) -> Command<Message> {Command::none()}

    fn save_updated_data(accounts: Vec<EntityAccount>, app:&'a mut App) {
        app.app_data.db.update_accounts(accounts.as_slice())
            .unwrap_or_else(|err| {
                debug_println!("Unable to update accounts: {err}");
            });
        for account in accounts {
            app.app_data.db.update_fungibles_for_account(&account.fungibles, &account.address)
                .unwrap_or_else(|err| {
                    debug_println!(
                        "{}:{} Unable to update fungibles: {err}",
                        module_path!(),
                        line!()
                    );
                });
            for fungible in account.fungibles.0 {
                if let Some(icon) = fungible.icon {
                    app.appview.resource_icons.entry(fungible.address)
                        .and_modify(|handle| *handle = icon.handle())
                        .or_insert(icon.handle());
                }
            }

            app.app_data.db.update_non_fungibles_for_account(&account.non_fungibles, &account.address)
                .unwrap_or_else(|err| {
                    debug_println!(
                        "{}:{} Unable to update non fungible: {err}",
                        module_path!(),
                        line!()
                    )
                });

            for non_fungible in account.non_fungibles.0 {
                if let Some(icon) = non_fungible.icon {
                    app.appview.resource_icons.entry(non_fungible.address)
                        .and_modify(|handle| *handle = icon.handle())
                        .or_insert(icon.handle());
                }
            }
            
        }
    }
}
