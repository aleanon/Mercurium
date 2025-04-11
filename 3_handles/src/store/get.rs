use deps_two::*;

use std::collections::HashMap;

use bytes::Bytes;
use debug_print::debug_println;
use store::{AppDataDb, DbError, IconsDb};
use types::address::ResourceAddress;
use types::{collections::AppdataFromDisk, AppError, Network};

pub async fn accounts_and_resources(network: Network) -> Result<AppdataFromDisk, DbError> {
    let Some(db) = AppDataDb::get(network) else {
        return Err(DbError::DatabaseNotLoaded);
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

    Ok(AppdataFromDisk {
        accounts,
        resources,
        fungible_assets,
        non_fungible_assets,
    })
}

pub async fn resource_icons(
    network: Network,
) -> Result<(Network, HashMap<ResourceAddress, Bytes>), AppError> {
    let Some(icon_cache) = IconsDb::get(network) else {
        return Err(AppError::Fatal("Icon cache not initialized".to_owned()));
    };

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
            let bytes = Bytes::from_owner(data);

            (resource_address, bytes)
        })
        .collect::<HashMap<ResourceAddress, Bytes>>();

    Ok((network, icons))
}
