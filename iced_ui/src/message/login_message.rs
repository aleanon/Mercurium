use core::net;
use std::collections::HashMap;

use debug_print::debug_println;
use iced::{futures::SinkExt, widget::image::Handle, Command};
use store::{Db, DbError, IconCache};
use types::{AccountsAndResources, Action, AppError, ResourceAddress};
use zeroize::Zeroize;

use crate::{app::AppState, App};

use super::{update_message::Update, Message};

#[derive(Debug, Clone)]
pub enum LoginMessage {
    TextInputChanged(String),
    Login,
}

impl Into<Message> for LoginMessage {
    fn into(self) -> Message {
        Message::Login(self)
    }
}

impl<'a> LoginMessage {
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();

        match self {
            LoginMessage::TextInputChanged(mut string) => {
                if let AppState::Locked(ref mut loginscreen) = app.app_state {
                    loginscreen.password.clear();
                    loginscreen.password.push_str(string.as_str());
                    string.zeroize()
                }
            }
            LoginMessage::Login => {
                if let AppState::Locked(ref login) = app.app_state {
                    // let salt
                    let (key, _salt) = login.password.derive_new_db_encryption_key().unwrap();
                    //take the password, verify and create encryption key and decrypt the database

                    if let Err(err) = app.login() {
                        match err {
                            AppError::Fatal(_) => app.app_state = AppState::Error(err.to_string()),
                            AppError::NonFatal(_) => { /*In app notification*/ }
                        }
                    };

                    // if let Err(err) = app.action_tx.send(Action::LoadDatabase(key)) {
                    //     app.state = State::Error(AppError::Fatal(Box::new(err)))
                    // }

                    #[cfg(not(feature = "noupdate"))]
                    let update_accounts = {
                        let network = app.app_data.settings.network;
                        let db = Db::load(network).unwrap();
                        Command::perform(
                            async move {
                                handles::radix_dlt::updates::update_all_accounts(network.into(), db)
                                    .await
                            },
                            |accounts_update| {
                                #[cfg(debug_assertions)]
                                for account_update in &accounts_update.account_updates {
                                    debug_println!("Found {} new fungibles and {} new non_fungibles for account: {}", account_update.fungibles.len(), account_update.non_fungibles.len(), account_update.account.address.as_str());
                                }
                                debug_println!(
                                    "Found {} new resources",
                                    accounts_update.new_resources.len()
                                );

                                Message::Update(super::update_message::Update::Accounts(
                                    accounts_update,
                                ))
                            },
                        )
                    };

                    let get_accounts_and_resources = {
                        let network = app.app_data.settings.network;
                        Command::perform(
                            async move {
                                let db = Db::load(network)?;
                                let accounts = db.get_accounts().unwrap_or_else(|err| {
                                    debug_println!("Failed to retrieve accounts: {}", err);
                                    HashMap::new()
                                });
                                let resources = db.get_all_resources().unwrap_or_else(|err| {
                                    debug_println!("Failed to retrieve resources: {}", err);
                                    HashMap::new()
                                });
                                let fungible_assets = db
                                    .get_all_fungible_assets_set_per_account()
                                    .unwrap_or_else(|err| {
                                        debug_println!(
                                            "Failed to retrieve fungible assets: {}",
                                            err
                                        );
                                        HashMap::new()
                                    });
                                let non_fungible_assets = db
                                    .get_all_non_fungible_assets_set_per_account()
                                    .unwrap_or_else(|err| {
                                        debug_println!(
                                            "Failed to retrieve non fungible assets: {}",
                                            err
                                        );
                                        HashMap::new()
                                    });

                                Ok::<_, DbError>(AccountsAndResources {
                                    accounts,
                                    resources,
                                    fungible_assets,
                                    non_fungible_assets,
                                })
                            },
                            |result| match result {
                                Ok(accounts_and_resources) => {
                                    debug_println!(
                                        "Retrieved {} accounts from disk",
                                        accounts_and_resources.accounts.len()
                                    );
                                    #[cfg(debug_assertions)]
                                    for account in &accounts_and_resources.accounts {
                                        if let Some(fungibles) =
                                            accounts_and_resources.fungible_assets.get(&account.0)
                                        {
                                            debug_println!(
                                                "Retrieved {} fungible assets for account: {}",
                                                fungibles.len(),
                                                &account.1.name
                                            );
                                        }
                                        if let Some(non_fungibles) = accounts_and_resources
                                            .non_fungible_assets
                                            .get(&account.0)
                                        {
                                            debug_println!(
                                                "Retrieved {} non fungible assets for account: {}",
                                                non_fungibles.len(),
                                                &account.1.name
                                            );
                                        }
                                    }
                                    debug_println!(
                                        "Retrieved {} resources from disk",
                                        accounts_and_resources.resources.len()
                                    );

                                    Message::Update(
                                        super::update_message::Update::AccountsAndResources(
                                            accounts_and_resources,
                                        ),
                                    )
                                }
                                Err(err) => {
                                    debug_println!("Error when opening database: {}", err);
                                    Message::None
                                }
                            },
                        )
                    };

                    let get_resource_icons = {
                        let network = app.app_data.settings.network;
                        Command::perform(
                            async move {
                                let icon_cache = IconCache::load(network)
                                    .await
                                    .map_err(|err| AppError::Fatal(Box::new(err)))?;

                                let icons_data = icon_cache
                                    .get_all_resource_icons()
                                    .await
                                    .unwrap_or_else(|err| {
                                        debug_println!(
                                            "Failed to retrieve resource icons: {}",
                                            err
                                        );
                                        HashMap::new()
                                    });

                                Ok::<_, AppError>(
                                    icons_data
                                        .into_iter()
                                        .map(|(resource_address, data)| {
                                            let handle = Handle::from_memory(data);

                                            (resource_address, handle)
                                        })
                                        .collect::<HashMap<ResourceAddress, Handle>>(),
                                )
                            },
                            |result| match result {
                                Ok(icons) => Message::Update(Update::Icons(icons)),
                                Err(err) => {
                                    debug_println!("Error when loading icon cache: {}", err);
                                    Message::None
                                }
                            },
                        )
                    };

                    command = Command::batch([
                        #[cfg(not(feature = "noupdate"))]
                        update_accounts,
                        get_accounts_and_resources,
                        get_resource_icons,
                    ])
                }
            }
        }
        command
    }
}
