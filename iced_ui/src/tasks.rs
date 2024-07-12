// use iced::Task;
// use store::AsyncDb;
// use types::{crypto::Password, AppError, Network};

// use crate::app::AppMessage;

// pub fn initial_login(password: Password, network: Network) -> Task<AppMessage> {
//     Task::perform(async move {
//         let salt = handles::credentials::get_db_encryption_salt()?;

//         let key = password.derive_db_encryption_key_from_salt(&salt);

//         let db = AsyncDb::load(network, key).await?;

//     }, f)

//     let load_db = {
//         Task::perform(async move {
//         }, f)
//     }

//     if let Some(db) = self.app_data.db.as_ref() {
//         let key_hash = {
//             let AppState::Locked(loginscreen) = &self.app_state else {
//                 return Err(AppError::Fatal(
//                     "Calling unlock when not in locked state".to_string(),
//                 ));
//             };

//             loginscreen
//                 .password()
//                 .derive_db_encryption_key_hash_from_salt(&salt)
//         };

//         let target_hash = db
//             .get_db_password_hash()
//             .map_err(|err| AppError::Fatal(err.to_string()))?;

//         if key_hash == target_hash {
//             self.app_state = AppState::Unlocked;
//             Ok(())
//         } else {
//             if let AppState::Locked(loginscreen) = &mut self.app_state {
//                 loginscreen.notification = "Wrong password".to_string();
//                 Ok(())
//             } else {
//                 return Err(AppError::Fatal(
//                     "Called login when not in Locked state".to_string(),
//                 ));
//             }
//         }
//     } else {
//         let key = {
//             let AppState::Locked(loginscreen) = &self.app_state else {
//                 return Err(AppError::Fatal(
//                     "Calling unlock when not in locked state".to_string(),
//                 ));
//             };

//             loginscreen
//                 .password()
//                 .derive_db_encryption_key_from_salt(&salt)
//         };

//         Db::load(self.app_data.settings.network, &key)
//             .map_err(|err| AppError::Fatal(err.to_string()))
//             .and_then(|db| {
//                 self.app_data.db = Some(db);
//                 self.app_state = AppState::Unlocked;
//                 Ok(())
//             })
//     }
// }

use std::collections::HashMap;

use debug_print::debug_println;
use iced::{widget::image::Handle, Task};
use store::{AsyncDb, DbError, IconCache};
use types::{AccountsAndResources, AppError, Network, ResourceAddress};

use crate::{app::AppMessage, task_response};

pub fn update_accounts(network: Network) -> Task<AppMessage> {
    Task::perform(
        async move {
            let db = AsyncDb::get(network)
                .await
                .ok_or(AppError::Fatal("Database not initialized".to_string()))?;
            Ok(handles::radix_dlt::updates::update_all_accounts(network, db).await)
        },
        |result| match result {
            Ok(accounts_update) => task_response::Message::AccountsUpdated(accounts_update).into(),
            Err(err) => task_response::Message::Error(err).into(),
        },
    )
}

pub fn get_accounts_and_resources_from_disk(network: Network) -> Task<AppMessage> {
    Task::perform(
        async move {
            let Some(db) = AsyncDb::get(network).await else {
                return Err(DbError::DatabaseNotInitialized);
            };
            let accounts = db.get_accounts().await.unwrap_or_else(|err| {
                debug_println!("Failed to retrieve accounts: {}", err);
                HashMap::new()
            });
            let resources = db.get_all_resources().await.unwrap_or_else(|err| {
                debug_println!("Failed to retrieve resources: {}", err);
                HashMap::new()
            });
            let fungible_assets = db
                .get_all_fungible_assets_per_account()
                .await
                .unwrap_or_else(|err| {
                    debug_println!("Failed to retrieve fungible assets: {}", err);
                    HashMap::new()
                });
            let non_fungible_assets = db
                .get_all_non_fungible_assets_per_account()
                .await
                .unwrap_or_else(|err| {
                    debug_println!("Failed to retrieve non fungible assets: {}", err);
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
                    if let Some(fungibles) = accounts_and_resources.fungible_assets.get(&account.0)
                    {
                        debug_println!(
                            "Retrieved {} fungible assets for account: {}",
                            fungibles.len(),
                            &account.1.name
                        );
                    }
                    if let Some(non_fungibles) =
                        accounts_and_resources.non_fungible_assets.get(&account.0)
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

                AppMessage::TaskResponse(task_response::Message::AccountsAndResources(
                    accounts_and_resources,
                ))
            }
            Err(err) => {
                debug_println!("Error when opening database: {}", err);
                AppMessage::None
            }
        },
    )
}

pub fn get_resource_icons_from_disk(network: Network) -> Task<AppMessage> {
    Task::perform(
        async move {
            let icon_cache = IconCache::load(network)
                .await
                .map_err(|err| AppError::Fatal(err.to_string()))?;

            let icons_data = icon_cache
                .get_all_resource_icons()
                .await
                .unwrap_or_else(|err| {
                    debug_println!("Failed to retrieve resource icons: {}", err);
                    HashMap::new()
                });

            Ok::<_, AppError>(
                icons_data
                    .into_iter()
                    .map(|(resource_address, data)| {
                        let handle = Handle::from_bytes(data);

                        (resource_address, handle)
                    })
                    .collect::<HashMap<ResourceAddress, Handle>>(),
            )
        },
        |result| match result {
            Ok(icons) => AppMessage::TaskResponse(task_response::Message::Icons(icons)),
            Err(err) => {
                debug_println!("Error when loading icon cache: {}", err);
                AppMessage::None
            }
        },
    )
}

pub fn initial_login_tasks(network: Network) -> Task<AppMessage> {
    Task::batch([
        #[cfg(not(feature = "noupdate"))]
        update_accounts(network),
        get_accounts_and_resources_from_disk(network),
        get_resource_icons_from_disk(network),
    ])
}
