use std::collections::{BTreeSet, HashMap};

use debug_print::debug_println;

use iced::widget::image::Handle;
use iced::Command;

use store::{Db, DbError};
use types::{assets::FungibleAsset, AccountsAndResources, AccountsUpdate, ResourceAddress};

use crate::{app::AppMessage, App};

// Uses a newtype wrapper because the Update type is used in the backend crate and has to be defined in the types crate

#[derive(Debug, Clone)]
pub enum Update {
    Accounts(AccountsUpdate),
    AccountsAndResources(AccountsAndResources),
    Icons(HashMap<ResourceAddress, Handle>),
}

impl Into<AppMessage> for Update {
    fn into(self) -> AppMessage {
        AppMessage::Update(self)
    }
}

impl<'a> Update {
    pub fn update(self, app: &'a mut App) -> Command<AppMessage> {
        match self {
            Self::Accounts(accounts_update) => {
                Self::process_updated_accounts_and_resources(accounts_update, app)
            }
            Self::Icons(icons) => Self::store_icons_in_app_data(icons, app),
            Self::AccountsAndResources(accounts_and_resources) => {
                Self::place_accounts_and_resources_in_memory(accounts_and_resources, app)
            }
        }
    }

    fn process_updated_accounts_and_resources(
        accounts_update: AccountsUpdate,
        app: &'a mut App,
    ) -> Command<AppMessage> {
        for account_update in &accounts_update.account_updates {
            match app
                .app_data
                .fungibles
                .get_mut(&account_update.account.address)
            {
                Some(fungibles) => {
                    for (_, asset) in &account_update.fungibles {
                        fungibles.replace(asset.clone());
                    }
                }
                None => {
                    let updated_fungibles = account_update
                        .fungibles
                        .iter()
                        .map(|(_, asset)| asset.clone())
                        .collect::<BTreeSet<FungibleAsset>>();
                    app.app_data
                        .fungibles
                        .insert(account_update.account.address.clone(), updated_fungibles);
                }
            }

            match app
                .app_data
                .non_fungibles
                .get_mut(&account_update.account.address)
            {
                Some(non_fungibles) => {
                    for (_, asset) in &account_update.non_fungibles {
                        non_fungibles.replace(asset.clone());
                    }
                }
                None => {
                    let updated_non_fungibles = account_update
                        .non_fungibles
                        .iter()
                        .map(|(_, asset)| asset.clone())
                        .collect::<BTreeSet<_>>();
                    app.app_data.non_fungibles.insert(
                        account_update.account.address.clone(),
                        updated_non_fungibles,
                    );
                }
            }

            match app
                .app_data
                .accounts
                .get_mut(&account_update.account.address)
            {
                Some(account) => {
                    account.balances_last_updated = account_update.account.balances_last_updated;
                }
                None => {
                    app.app_data.accounts.insert(
                        account_update.account.address.clone(),
                        account_update.account.clone(),
                    );
                }
            }
        }
        app.app_data
            .resources
            .extend(accounts_update.new_resources.clone());

        let download_icons = {
            let icon_urls = accounts_update.icon_urls;
            let network = app.app_data.settings.network;
            Command::perform(
                async move {
                    handles::image::download::download_resize_and_store_resource_icons(
                        icon_urls, network,
                    )
                    .await
                },
                |icons| Update::Icons(icons).into(),
            )
        };

        let save_accounts_and_resources_to_disk = {
            let account_updates = accounts_update.account_updates;
            let new_resources = accounts_update
                .new_resources
                .into_iter()
                .map(|(_, resource)| resource)
                .collect::<Vec<_>>();
            let network = app.app_data.settings.network;
            Command::perform(
                async move {
                    let mut db = Db::load(network)?;
                    db.upsert_resources(new_resources.as_slice())?;
                    for account_update in account_updates {
                        db.upsert_account(&account_update.account)?;

                        debug_println!(
                            "Attempting to save {} fungibles to disk",
                            account_update.fungibles.len()
                        );

                        let fungibles = account_update
                            .fungibles
                            .into_iter()
                            .map(|(_, fungible)| fungible)
                            .collect::<Vec<_>>();
                        db.upsert_fungible_assets_for_account(
                            &account_update.account.address,
                            &fungibles,
                        )?;

                        debug_println!(
                            "Attempting to save {} non fungibles to disk",
                            account_update.non_fungibles.len()
                        );
                        let non_fungibles = account_update
                            .non_fungibles
                            .into_iter()
                            .map(|(_, non_fungible)| non_fungible)
                            .collect::<Vec<_>>();
                        db.upsert_non_fungible_assets_for_account(
                            &account_update.account.address,
                            &non_fungibles,
                        )?;
                    }
                    Ok::<_, DbError>(())
                },
                |result| match result {
                    Ok(_) => AppMessage::None,
                    Err(err) => {
                        debug_println!("Failed to save accounts and resources to disk: {}", err);
                        AppMessage::None
                    }
                },
            )
        };

        Command::batch([download_icons, save_accounts_and_resources_to_disk])
    }

    fn store_icons_in_app_data(
        icons: HashMap<ResourceAddress, Handle>,
        app: &'a mut App,
    ) -> Command<AppMessage> {
        for (resource_address, icon) in icons {
            app.app_data.resource_icons.insert(resource_address, icon);
        }

        Command::none()
    }

    fn place_accounts_and_resources_in_memory(
        accounts_and_resources: AccountsAndResources,
        app: &'a mut App,
    ) -> Command<AppMessage> {
        app.app_data.accounts = accounts_and_resources.accounts;
        app.app_data.resources = accounts_and_resources.resources;
        app.app_data.fungibles = accounts_and_resources.fungible_assets;
        app.app_data.non_fungibles = accounts_and_resources.non_fungible_assets;

        Command::none()
    }
}

// #[derive(Debug, Clone)]
// pub struct UpdateMessage(pub(crate) Update);

// impl<'a> UpdateMessage {
//     pub fn process(self, app: &'a mut App) -> Command<Message> {
//         let mut command = Command::none();
//         match self.0 {
//             Update::Sender(action_tx) => app.app_data.backend_sender = action_tx,
//             Update::Icons(icons) => Self::store_icons_in_cache(icons, app),
//             Update::Accounts(accounts) => Self::save_updated_data(accounts, app),
//             Update::DatabaseLoaded => command = Self::send_update_all_request(app),

//             _ => {}
//         }
//         command
//     }

//     fn store_icons_in_cache(icons: HashMap<ResourceAddress, Handle>, app: &'a mut App) {
//         debug_println!(
//             "{}:{} Received {}icons:",
//             module_path!(),
//             line!(),
//             icons.len()
//         );
//         app.app_data.resource_icons = icons;
//     }

//     #[cfg(not(feature = "noupdate"))]
//     fn send_update_all_request(app: &'a mut App) -> Command<Message> {
//         let mut channel = app.app_data.backend_sender.clone();
//         Command::perform(async move { channel.send(Action::UpdateAll).await }, |_| {
//             Message::None
//         })
//     }

//     #[cfg(feature = "noupdate")]
//     fn send_update_all_request(app: &'a mut App) -> Command<Message> {
//         Command::none()
//     }

//     fn save_updated_data(accounts: Vec<EntityAccount>, app: &'a mut App) {
//         app.app_data
//             .db
//             .update_accounts(accounts.as_slice())
//             .unwrap_or_else(|err| {
//                 debug_println!("Unable to update accounts: {err}");
//             });
//         for account in accounts {
//             app.app_data
//                 .db
//                 .update_fungibles_for_account(&account.fungibles, &account.address)
//                 .unwrap_or_else(|err| {
//                     debug_println!(
//                         "{}:{} Unable to update fungibles: {err}",
//                         module_path!(),
//                         line!()
//                     );
//                 });
//             for fungible in account.fungibles.0 {
//                 if let Some(icon) = fungible.icon {
//                     app.appview
//                         .resource_icons
//                         .entry(fungible.address)
//                         .and_modify(|handle| *handle = icon.handle())
//                         .or_insert(icon.handle());
//                 }
//             }

//             app.app_data
//                 .db
//                 .update_non_fungibles_for_account(&account.non_fungibles, &account.address)
//                 .unwrap_or_else(|err| {
//                     debug_println!(
//                         "{}:{} Unable to update non fungible: {err}",
//                         module_path!(),
//                         line!()
//                     )
//                 });

//             for non_fungible in account.non_fungibles.0 {
//                 if let Some(icon) = non_fungible.icon {
//                     app.appview
//                         .resource_icons
//                         .entry(non_fungible.address)
//                         .and_modify(|handle| *handle = icon.handle())
//                         .or_insert(icon.handle());
//                 }
//             }
//         }
//     }
// }
