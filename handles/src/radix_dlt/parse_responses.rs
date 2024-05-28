use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use bytes::Bytes;
use image::imageops::FilterType;
use radix_gateway_sdk::generated::model::{
    FungibleResourcesCollection, FungibleResourcesCollectionItem, NonFungibleResourcesCollection,
    StateEntityDetailsResponse,
};
use types::assets::{AssetId, FungibleAsset, NewAssets, NewNonFungibles, NonFungibleAsset};
use types::response_models::accounts_details::{
    AccountsDetails, NonFungibleResourceVaultAggregated,
};
use types::response_models::{
    Entity, FungibleResource, FungibleResourceVaultAggregated, NonFungibleResource,
    TransactionsResponse,
};
use types::{non_fungibles::NonFungible, Account};
use types::{
    AccountAddress, AppPath, Decimal, Fungible, Icon, MetaData, NFIDs, RadixDecimal,
    ResourceAddress, NFID,
};

use crate::filesystem::resize_image::resize_image;

async fn parse_fungible_response(
    fungible_resources: Arc<HashMap<String, FungibleResource>>,
    fungible: Entity,
) -> Option<Fungible> {
    let (last_updated, amount) = match fungible_resources.get(&*fungible.address) {
        Some(fungible_resource) => {
            let mut amount = RadixDecimal::ZERO;
            let mut last_updated = 0;
            for vault in &fungible_resource.vaults.items {
                amount +=
                    RadixDecimal::from_str(&vault.amount).unwrap_or_else(|_| RadixDecimal::ZERO);
                if last_updated < vault.last_updated_at_state_version {
                    last_updated = vault.last_updated_at_state_version
                }
            }
            (last_updated, amount.into())
        }
        None => (0, RadixDecimal::ZERO.into()),
    };

    let address = ResourceAddress::from_str(&fungible.address).ok()?;

    let mut name = None;
    let mut symbol = None;
    let mut description = None;
    let mut icon_url = None;
    let mut metadata = MetaData::new();
    let total_supply = fungible.details.total_supply.unwrap_or(String::new());

    for item in fungible.metadata.items {
        match &*item.key {
            "name" => name = item.value.typed.value,
            "symbol" => symbol = item.value.typed.value,
            "description" => description = item.value.typed.value,
            "icon_url" => icon_url = item.value.typed.value.filter(|value| value.len() != 0),
            _ => metadata.push(item.into()),
        }
    }

    let icon = get_icon(icon_url, &address).await;

    let fungible = Fungible {
        address,
        amount,
        total_supply,
        description,
        name: name.unwrap_or(String::new()),
        symbol: symbol.unwrap_or(String::new()),
        icon,
        last_updated_at_state_version: last_updated as i64,
        metadata,
    };
    Some(fungible)
}

/// Takes a map where the address is the key of the map, the `NonFungibleResource` is the response
/// model from for a NonFungible after a gateway request
async fn parse_non_fungible_response(
    non_fungible_resources: Arc<HashMap<String, NonFungibleResource>>,
    non_fungible: Entity,
) -> Option<NonFungible> {
    let (last_updated, nfids) = match non_fungible_resources.get(&*non_fungible.address) {
        Some(non_fungible_resource) => {
            let mut last_updated = 0;
            for vault in &non_fungible_resource.vaults.items {
                if last_updated < vault.last_updated_at_state_version {
                    last_updated = vault.last_updated_at_state_version
                }
            }
            let nfids = NFIDs::from(&non_fungible_resource.vaults);
            (last_updated, nfids)
        }
        None => (0, NFIDs::new()),
    };

    let address = ResourceAddress::from_str(&non_fungible.address).ok()?;

    let mut name = None;
    let mut symbol = None;
    let mut description = None;
    let mut icon_url = None;
    let mut metadata = MetaData::new();
    let _current_supply = non_fungible.details.total_supply.unwrap_or(String::new());

    for item in non_fungible.metadata.items {
        match &*item.key {
            "name" => name = item.value.typed.value,
            "symbol" => symbol = item.value.typed.value,
            "description" => description = item.value.typed.value,
            "icon_url" => icon_url = item.value.typed.value.filter(|value| value.len() != 0),
            _ => metadata.push(item.into()),
        }
    }

    let icon = get_icon(icon_url, &address).await;

    let non_fungible = NonFungible {
        address,
        description,
        name: name.unwrap_or(String::new()),
        symbol: symbol.unwrap_or(String::new()),
        icon,
        nfids,
        last_updated_at_state_version: last_updated as i64,
        metadata,
    };
    Some(non_fungible)
}

async fn get_icon(icon_url: Option<String>, resource_address: &ResourceAddress) -> Option<Icon> {
    let url = icon_url?;
    if let Ok(app_path) = AppPath::new() {
        let mut icon_path = app_path.icons_directory().clone();
        icon_path.push(resource_address.as_str());
        if icon_path.exists() {
            if let Ok(image) = image::open(&icon_path) {
                if let Some(resized) = resize_image(
                    &image,
                    NonZeroU32::new(50).unwrap(),
                    NonZeroU32::new(50).unwrap(),
                ) {
                    Some(Icon::new(Bytes::from(resized.buffer().to_vec())))
                } else
                //Could not resize image
                {
                    download_icon(&url, Some(&mut icon_path)).await
                }
            } else
            //Could not open image
            {
                download_icon(&url, Some(&mut icon_path)).await
            }
        } else
        //Icon path does not exist
        {
            download_icon(&url, Some(&mut icon_path)).await
        }
    } else
    //Unable to determine icons directory
    {
        download_icon(&url, None).await
    }
}

async fn download_icon(url: &String, icon_path: Option<&mut PathBuf>) -> Option<Icon> {
    let response = reqwest::get(url).await.ok()?;

    let bytes = response.bytes().await.ok()?;
    let reader = image::io::Reader::new(Cursor::new(&bytes));
    let with_guessed_format = reader.with_guessed_format().ok()?;
    let format = with_guessed_format.format()?;
    let image = with_guessed_format.decode().ok()?;

    if let Some(path) = icon_path {
        path.set_extension(crate::filesystem::image_extension::get_extension(&format));
        image.save_with_format(path, format).unwrap_or(());
    }

    let resized = image.resize(50, 50, FilterType::Lanczos3);
    let mut write_buffer = BufWriter::new(Cursor::new(Vec::new()));
    resized.write_to(&mut write_buffer, format).ok()?;

    let inner = write_buffer.into_inner().ok()?.into_inner();
    let icon = Icon::new(Bytes::from(inner));

    Some(icon)
}

// pub fn parse_transactions_response(response: TransactionsResponse) -> Result<Transaction> {}

pub fn update_assets_from_accounts_details_response(
    fungible_assets: &mut HashMap<AccountAddress, HashMap<ResourceAddress, FungibleAsset>>,
    non_fungible_assets: &mut HashMap<AccountAddress, HashMap<ResourceAddress, NonFungibleAsset>>,
    entity_response: StateEntityDetailsResponse,
) -> NewAssets {
    let mut new_assets = NewAssets::new();

    for account in entity_response.items {
        let account_address = match AccountAddress::from_str(&account.address) {
            Ok(address) => address,
            Err(_) => continue,
        };

        if let Some(fungible_resources_response) = account.fungible_resources {
            if let Some(fungible_assets_for_account) = fungible_assets.get_mut(&account_address) {
                update_fungible_assets_from_account_details_response(
                    &mut new_assets,
                    &account_address,
                    fungible_assets_for_account,
                    fungible_resources_response,
                )
            } else {
                let mut fungible_assets_for_account = HashMap::new();
                update_fungible_assets_from_account_details_response(
                    &mut new_assets,
                    &account_address,
                    &mut fungible_assets_for_account,
                    fungible_resources_response,
                );
                fungible_assets.insert(account_address.clone(), fungible_assets_for_account);
            }
        };

        if let Some(non_fungible_resources_response) = account.non_fungible_resources {
            if let Some(non_fungible_assets_for_account) =
                non_fungible_assets.get_mut(&account_address)
            {
                update_non_fungible_assets_from_account_details_response(
                    &mut new_assets,
                    &account_address,
                    non_fungible_assets_for_account,
                    non_fungible_resources_response,
                )
            } else {
                let mut non_fungible_assets_for_account = HashMap::new();
                update_non_fungible_assets_from_account_details_response(
                    &mut new_assets,
                    &account_address,
                    &mut non_fungible_assets_for_account,
                    non_fungible_resources_response,
                );
                non_fungible_assets.insert(account_address, non_fungible_assets_for_account);
            }
        }
    }
    new_assets
}

fn update_fungible_assets_from_account_details_response(
    new_assets: &mut NewAssets,
    account_address: &AccountAddress,
    assets: &mut HashMap<ResourceAddress, FungibleAsset>,
    fungible_resources_response: FungibleResourcesCollection,
) {
    for non_fungible_resource in fungible_resources_response.items {
        if let Ok(fungible_resource) =
            serde_json::from_value::<FungibleResourceVaultAggregated>(non_fungible_resource.0)
        {
            let resource_address =
                match ResourceAddress::from_str(&fungible_resource.resource_address) {
                    Ok(address) => address,
                    Err(_) => continue,
                };

            if let Some(asset) = assets.get_mut(&resource_address) {
                let (last_updated, amount) = fungible_resource.vaults.items.iter().fold(
                    (0, RadixDecimal::ZERO),
                    |(mut last_updated, amount), vault| {
                        let amount_parsed =
                            RadixDecimal::from_str(&vault.amount).unwrap_or(RadixDecimal::ZERO);
                        if last_updated < vault.last_updated_at_state_version {
                            last_updated = vault.last_updated_at_state_version;
                        }

                        (last_updated, amount + amount_parsed)
                    },
                );

                asset.last_updated = last_updated;
                asset.amount = amount.to_string();
            } else {
                new_assets.new_fungibles.insert(resource_address.clone());

                let symbol = fungible_resource
                    .explicit_metadata
                    .items
                    .into_iter()
                    .find_map(|metadataitem| {
                        if metadataitem.key == "symbol" {
                            Some(metadataitem.value.typed.value)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(String::new());

                let (last_updated, amount) = fungible_resource.vaults.items.iter().fold(
                    (0, RadixDecimal::ZERO),
                    |(mut last_updated, amount), vault| {
                        let amount_parsed =
                            RadixDecimal::from_str(&vault.amount).unwrap_or(RadixDecimal::ZERO);
                        if last_updated < vault.last_updated_at_state_version {
                            last_updated = vault.last_updated_at_state_version;
                        }

                        (last_updated, amount + amount_parsed)
                    },
                );

                let fungible_asset = FungibleAsset::new(
                    symbol,
                    &account_address,
                    amount.to_string(),
                    resource_address.clone(),
                    last_updated,
                );

                assets.insert(resource_address, fungible_asset);
            };
        }
    }
}

fn update_non_fungible_assets_from_account_details_response(
    new_assets: &mut NewAssets,
    account_address: &AccountAddress,
    assets: &mut HashMap<ResourceAddress, NonFungibleAsset>,
    non_fungible_resources_response: NonFungibleResourcesCollection,
) {
    for fungible_resource in non_fungible_resources_response.items {
        if let Ok(non_fungible_resource) =
            serde_json::from_value::<NonFungibleResourceVaultAggregated>(fungible_resource.0)
        {
            let resource_address =
                match ResourceAddress::from_str(&non_fungible_resource.resource_address) {
                    Ok(address) => address,
                    Err(_) => continue,
                };

            if let Some(asset) = assets.get_mut(&resource_address) {
                let mut newest_update = asset.last_updated;
                for vault in non_fungible_resource.vaults.items {
                    if vault.last_updated_at_state_version < asset.last_updated {
                        continue;
                    }

                    for nfid_string in vault.items {
                        let nfid = NFID::new(nfid_string);
                        if !asset.nfids.contains(&nfid) {
                            asset.nfids.insert(nfid.clone());
                            new_assets
                                .new_non_fungibles
                                .insert(&resource_address, nfid.get_id());
                        }
                    }

                    if vault.last_updated_at_state_version > newest_update {
                        newest_update = vault.last_updated_at_state_version;
                    }
                }
            } else {
                let symbol = non_fungible_resource
                    .explicit_metadata
                    .items
                    .into_iter()
                    .find_map(|metadataitem| {
                        if metadataitem.key == "symbol" {
                            Some(metadataitem.value.typed.value)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(String::new());

                let mut nfids = NFIDs::new();
                let mut last_updated = 0;
                for vault in non_fungible_resource.vaults.items {
                    if last_updated < vault.last_updated_at_state_version {
                        last_updated = vault.last_updated_at_state_version;
                    }
                    for nfid in vault.items {
                        let nfid = NFID::new(nfid);
                        nfids.insert(nfid.clone());
                        new_assets
                            .new_non_fungibles
                            .insert(&resource_address, nfid.get_id());
                    }
                }

                let non_fungible_asset = NonFungibleAsset::new(
                    symbol,
                    &account_address,
                    nfids,
                    resource_address.clone(),
                    last_updated,
                );

                assets.insert(resource_address, non_fungible_asset);
            }
        }
    }
}
