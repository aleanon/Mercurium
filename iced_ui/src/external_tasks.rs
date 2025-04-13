use deps::*;

use debug_print::debug_println;
use iced::Task;
use types::{collections::AppdataFromDisk, Network};

use crate::{app::AppMessage, external_task_response};

pub fn update_all_accounts(network: Network) -> Task<AppMessage> {
    Task::perform(
        async move { handles::radix_dlt::updates::update_all_accounts(network).await },
        |result| match result {
            Ok(accounts_update) => {
                external_task_response::Message::AccountsUpdated(accounts_update).into()
            }
            Err(err) => external_task_response::Message::Error(err).into(),
        },
    )
}

pub fn get_app_data_from_disk(network: Network) -> Task<AppMessage> {
    Task::perform(
        async move { handles::store::get::accounts_and_resources(network).await },
        |result| match result {
            Ok(accounts_and_resources) => {
                #[cfg(debug_assertions)]
                print_app_data_info(&accounts_and_resources);

                AppMessage::TaskResponse(external_task_response::Message::AccountsAndResources(
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

#[cfg(debug_assertions)]
fn print_app_data_info(accounts_and_resources: &AppdataFromDisk) {
    println!(
        "Retrieved {} accounts from disk",
        accounts_and_resources.accounts.len()
    );

    for account in &accounts_and_resources.accounts {
        if let Some(fungibles) = accounts_and_resources.fungible_assets.get(&account.0) {
            println!(
                "Retrieved {} fungible assets for account: {}",
                fungibles.len(),
                &account.1.name
            );
        }
        if let Some(non_fungibles) = accounts_and_resources.non_fungible_assets.get(&account.0) {
            println!(
                "Retrieved {} non fungible assets for account: {}",
                non_fungibles.len(),
                &account.1.name
            );
        }
    }
    println!(
        "Retrieved {} resources from disk",
        accounts_and_resources.resources.len()
    );
}

pub fn get_resource_icons_from_disk(network: Network) -> Task<AppMessage> {
    Task::perform(
        async move { handles::store::get::resource_icons(network).await },
        |result| match result {
            Ok(icons) => AppMessage::TaskResponse(external_task_response::Message::Icons(icons)),
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
        update_all_accounts(network),
        get_app_data_from_disk(network),
        get_resource_icons_from_disk(network),
    ])
}
