use std::collections::HashMap;

use debug_print::debug_println;
use iced::{widget::image::Handle, Task};
use store::{AsyncDb, DbError, IconCache};
use types::{address::ResourceAddress, collections::AppdataFromDisk, AppError, Network};

use crate::{app::AppMessage, task_response};

pub fn update_accounts(network: Network) -> Task<AppMessage> {
    Task::perform(
        async move {
            let db = AsyncDb::get(network)
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
            let Some(db) = AsyncDb::get(network) else {
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

            Ok::<_, DbError>(AppdataFromDisk {
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

            let icons = icons_data
                .into_iter()
                .map(|(resource_address, data)| {
                    let handle = Handle::from_bytes(data);

                    (resource_address, handle)
                })
                .collect::<HashMap<ResourceAddress, Handle>>();
            Ok::<_, AppError>((network, icons))
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
