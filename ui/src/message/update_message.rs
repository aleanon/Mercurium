use std::collections::HashMap;

use debug_print::debug_println;
use iced::futures::channel::mpsc::Sender;
use iced::widget::image::Handle;
use iced::Command;
use iced::futures::SinkExt;


use types::{Action, EntityAccount, ResourceAddress, Update};

use crate::{message::Message, App};


#[derive(Debug, Clone)]
pub struct UpdateMessage(pub(crate)Update);


impl<'a> UpdateMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self.0 {
            Update::Sender(action_tx) => Self::store_channel_in_app_state(action_tx, app),
            Update::Icons(icons) => Self::store_icons_in_cache(icons, app), 
            Update::Accounts(accounts) => Self::save_updated_data_to_disk(accounts, app),
            Update::DatabaseLoaded => Self::send_update_all_request(app),
            _ => {Command::none()}
        }
    }

    fn store_channel_in_app_state(action_tx: Sender<Action>, app: &'a mut App) -> Command<Message> {
        app.action_tx = Some(action_tx);

        Command::none()
    }

    fn store_icons_in_cache(icons: HashMap<ResourceAddress, Handle>, app: &'a mut App) -> Command<Message> {
       debug_println!(
            "{}:{} Received {}icons:",
            module_path!(),
            line!(),
            icons.len()
        );
        app.appview.resource_icons = icons; 

        Command::none()
    }

    fn send_update_all_request(app: &'a mut App) -> Command<Message>{
        if let Some(ref channel) = app.action_tx {
                let mut channel = channel.clone();
                Command::perform(
                    async move { channel.send(Action::UpdateAll).await },
                    |_| Message::None,
                )
        } else {Command::none()}
    }

    fn save_updated_data_to_disk(accounts: Vec<EntityAccount>, app:&'a mut App) -> Command<Message> {
         match app.db {
            Some(ref mut db) => {
                db.update_accounts(accounts.as_slice())
                    .unwrap_or_else(|err| {
                        debug_println!("Unable to update accounts: {err}");
                    });
                for account in accounts {
                    db.update_fungibles_for_account(&account.fungibles, &account.address)
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

                    if let Some(non_fungibles) = account.non_fungibles {
                        db.update_non_fungibles_for_account(&non_fungibles, &account.address)
                            .unwrap_or_else(|err| {
                                debug_println!(
                                    "{}:{} Unable to update non fungible: {err}",
                                    module_path!(),
                                    line!()
                                )
                            });

                        for non_fungible in non_fungibles.0 {
                            if let Some(icon) = non_fungible.icon {
                                app.appview.resource_icons.entry(non_fungible.address)
                                    .and_modify(|handle| *handle = icon.handle())
                                    .or_insert(icon.handle());
                            }
                        }
                    }
                }
            }
            None => {
                debug_println!("{}:{}No database found", module_path!(), line!())
            }
        };
        Command::none()
    }
}
