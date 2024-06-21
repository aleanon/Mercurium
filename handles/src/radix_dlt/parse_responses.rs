use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;

use bytes::Bytes;
use debug_print::debug_println;
use image::imageops::FilterType;
use radix_gateway_sdk::generated::model::{
    StateEntityDetailsResponseItem, StateEntityFungiblesPageResponse,
    StateEntityNonFungiblesPageResponse, StateNonFungibleDataResponse,
};
use types::app_path::AppPath;
use types::assets::{FungibleAsset, NonFungibleAsset};
use types::response_models::non_fungible_id_data::NFIdData;
use types::response_models::{
    FungibleCollectionItemGlobal, MetaDataStringArrayValue, MetaDataStringValue,
    NonFungibleCollectionItemVaultAggregated, ResourceDetails,
};
use types::{app_path, debug_info, Resource};
use types::{AccountAddress, AppPathInner, Icon, NFIDs, ResourceAddress, NFID};

use crate::filesystem::resize_image::resize_image;

// pub fn parse_transactions_response(response: TransactionsResponse) -> Result<Transaction> {}

pub struct ParseResponseError;

// pub fn parse_account_details_response_and_update_assets(
//     account_address: AccountAddress,
//     fungible_assets: &mut HashMap<ResourceAddress, FungibleAsset>,
//     non_fungible_assets: &mut HashMap<ResourceAddress, NonFungibleAsset>,
//     entity_response: StateEntityDetailsResponseItem,
// ) -> NewAssets {
//     let mut new_assets = NewAssets::new();

//     if let Some(fungible_resources_response) = entity_response.fungible_resources {
//         parse_account_details_response_and_update_fungible_assets(
//             &mut new_assets,
//             &account_address,
//             fungible_assets,
//             fungible_resources_response,
//         )
//     };

//     if let Some(non_fungible_resources_response) = entity_response.non_fungible_resources {
//         parse_account_details_response_and_update_non_fungible_assets(
//             &mut new_assets,
//             &account_address,
//             non_fungible_assets,
//             non_fungible_resources_response,
//         )
//     }

//     new_assets
// }

// fn parse_account_details_response_and_update_fungible_assets(
//     new_assets: &mut NewAssets,
//     account_address: &AccountAddress,
//     assets: &mut HashMap<ResourceAddress, FungibleAsset>,
//     fungible_resources_response: FungibleResourcesCollection,
// ) {
//     for fungible_resource in fungible_resources_response.items {
//         if let Ok(fungible_resource) =
//             serde_json::from_value::<FungibleResourceVaultAggregated>(fungible_resource.0)
//         {
//             let resource_address =
//                 match ResourceAddress::from_str(&fungible_resource.resource_address) {
//                     Ok(address) => address,
//                     Err(_) => continue,
//                 };
//             debug_println!("{}", resource_address.to_string());

//             if let Some(asset) = assets.get_mut(&resource_address) {
//                 let (last_updated, amount) = fungible_resource.vaults.items.iter().fold(
//                     (0, RadixDecimal::ZERO),
//                     |(mut last_updated, amount), vault| {
//                         let amount_parsed =
//                             RadixDecimal::from_str(&vault.amount).unwrap_or(RadixDecimal::ZERO);
//                         if last_updated < vault.last_updated_at_state_version {
//                             last_updated = vault.last_updated_at_state_version;
//                         }

//                         (last_updated, amount + amount_parsed)
//                     },
//                 );
//                 debug_println!("amount: {}", &amount);

//                 asset.amount = amount.to_string();
//             } else {
//                 new_assets.new_fungibles.insert(resource_address.clone());

//                 let symbol = fungible_resource
//                     .explicit_metadata
//                     .items
//                     .into_iter()
//                     .find_map(|metadataitem| {
//                         if metadataitem.key == "symbol" {
//                             Some(metadataitem.value.typed.value)
//                         } else {
//                             None
//                         }
//                     })
//                     .unwrap_or(String::new());

//                 let (last_updated, amount) = fungible_resource.vaults.items.iter().fold(
//                     (0, RadixDecimal::ZERO),
//                     |(mut last_updated, amount), vault| {
//                         let amount_parsed =
//                             RadixDecimal::from_str(&vault.amount).unwrap_or(RadixDecimal::ZERO);
//                         if last_updated < vault.last_updated_at_state_version {
//                             last_updated = vault.last_updated_at_state_version;
//                         }

//                         (last_updated, amount + amount_parsed)
//                     },
//                 );

//                 let fungible_asset = FungibleAsset::new(
//                     &account_address,
//                     amount.to_string(),
//                     resource_address.clone(),
//                 );

//                 assets.insert(resource_address, fungible_asset);
//             };
//         }
//     }
// }

// fn parse_account_details_response_and_update_non_fungible_assets(
//     new_assets: &mut NewAssets,
//     account_address: &AccountAddress,
//     assets: &mut HashMap<ResourceAddress, NonFungibleAsset>,
//     non_fungible_resources_response: NonFungibleResourcesCollection,
// ) {
//     for fungible_resource in non_fungible_resources_response.items {
//         if let Ok(non_fungible_resource) =
//             serde_json::from_value::<NonFungibleResourceVaultAggregated>(fungible_resource.0)
//         {
//             let resource_address =
//                 match ResourceAddress::from_str(&non_fungible_resource.resource_address) {
//                     Ok(address) => address,
//                     Err(_) => continue,
//                 };

//             if let Some(asset) = assets.get_mut(&resource_address) {
//                 for vault in non_fungible_resource.vaults.items {
//                     for nfid_string in vault.items {
//                         let nfid = NFID::new(nfid_string);
//                         if !asset.nfids.contains(&nfid) {
//                             asset.nfids.push(nfid.clone());
//                             new_assets
//                                 .new_non_fungibles
//                                 .insert(&resource_address, nfid.get_id());
//                         }
//                     }
//                 }
//             } else {
//                 let symbol = non_fungible_resource
//                     .explicit_metadata
//                     .items
//                     .into_iter()
//                     .find_map(|metadataitem| {
//                         if metadataitem.key == "symbol" {
//                             Some(metadataitem.value.typed.value)
//                         } else {
//                             None
//                         }
//                     })
//                     .unwrap_or(String::new());

//                 let mut nfids = NFIDs::new();
//                 let mut last_updated = 0;
//                 for vault in non_fungible_resource.vaults.items {
//                     if last_updated < vault.last_updated_at_state_version {
//                         last_updated = vault.last_updated_at_state_version;
//                     }
//                     for nfid in vault.items {
//                         let nfid = NFID::new(nfid);
//                         nfids.push(nfid.clone());
//                         new_assets
//                             .new_non_fungibles
//                             .insert(&resource_address, nfid.get_id());
//                     }
//                 }

//                 let non_fungible_asset =
//                     NonFungibleAsset::new(&account_address, nfids, resource_address.clone());

//                 assets.insert(resource_address, non_fungible_asset);
//             }
//         }
//     }
// }

/// Returns two tuples, first with the resource_address and resource and the second wit the resource_address and url to the resources icon
/// Returns None if the resource_address conversion failes.
pub fn parse_resource_details_response(
    response: StateEntityDetailsResponseItem,
) -> Option<(ResourceAddress, (Resource, String))> {
    let resource_address = ResourceAddress::from_str(&response.address).ok()?;

    let (current_supply, divisibility) = response
        .details
        .and_then(|details| {
            serde_json::from_value::<ResourceDetails>(details.0)
                .and_then(|details| Ok((details.total_supply, details.divisibility)))
                .ok()
        })
        .unwrap_or((String::new(), None));

    let mut name = String::new();
    let mut symbol = String::new();
    let mut description = String::new();
    let mut icon_url = String::new();
    let mut tags: Vec<String> = Vec::new();

    for metadataitem in response.metadata.items {
        match metadataitem.key.as_str() {
            "name" => {
                if let Ok(value) =
                    serde_json::from_value::<MetaDataStringValue>(metadataitem.value.typed.0)
                {
                    name = value.value
                }
            }
            "symbol" => {
                if let Ok(value) =
                    serde_json::from_value::<MetaDataStringValue>(metadataitem.value.typed.0)
                {
                    symbol = value.value;
                }
            }
            "description" => {
                if let Ok(value) =
                    serde_json::from_value::<MetaDataStringValue>(metadataitem.value.typed.0)
                {
                    description = value.value;
                }
            }
            "icon_url" => {
                if let Ok(value) =
                    serde_json::from_value::<MetaDataStringValue>(metadataitem.value.typed.0)
                {
                    icon_url = value.value;
                }
            }
            "tags" => {
                if let Ok(value) =
                    serde_json::from_value::<MetaDataStringArrayValue>(metadataitem.value.typed.0)
                {
                    tags = value.values;
                }
            }
            _ => {}
        }
    }

    let resource = Resource {
        address: resource_address.clone(),
        name,
        symbol,
        description,
        current_supply,
        divisibility,
        tags: tags.into(),
    };

    Some((resource_address, (resource, icon_url)))
}

pub fn parse_fungible_balances_response(
    response: StateEntityFungiblesPageResponse,
    last_updated_at_state_version: i64,
    account_address: &AccountAddress,
) -> HashMap<ResourceAddress, FungibleAsset> {
    response
        .fungible_resources_collection
        .items
        .into_iter()
        .filter_map(|item| {
            let fungible = serde_json::from_value::<FungibleCollectionItemGlobal>(item.0)
                .inspect_err(|err| {
                    debug_println!("{}:{}", debug_info!("Failed to parse Json value"), err)
                })
                .ok()?;

            if fungible.last_updated_at_state_version < last_updated_at_state_version {
                return None;
            }

            let resource_address = ResourceAddress::from_str(&fungible.resource_address).ok()?;

            let asset =
                FungibleAsset::new(&account_address, fungible.amount, resource_address.clone());

            Some((resource_address, asset))
        })
        .collect()
}

/// Returns a `HashMap` with a typle value of String that represents the vault address
/// to later be able to get the nfids of the returned asset
pub fn parse_non_fungible_balances_response_without_nfids(
    response: StateEntityNonFungiblesPageResponse,
    last_updated_at_state_version: i64,
    account_address: &AccountAddress,
) -> HashMap<ResourceAddress, (String, NonFungibleAsset)> {
    response
        .non_fungible_resources_collection
        .items
        .into_iter()
        .filter_map(|item| {
            let mut collection_item =
                serde_json::from_value::<NonFungibleCollectionItemVaultAggregated>(item.0)
                    .inspect_err(|err| {
                        debug_println!("{}:{}", debug_info!("Failed to parse Json value"), err)
                    })
                    .ok()?;

            let resource_address =
                ResourceAddress::from_str(collection_item.resource_address.as_str()).ok()?;

            // This collection should always return one element
            if collection_item.vaults.items.len() > 0 {
                let vault = collection_item.vaults.items.remove(0);
                if vault.last_updated_at_state_version < last_updated_at_state_version {
                    return None;
                }

                let vault_address = vault.vault_address;

                let asset =
                    NonFungibleAsset::new(account_address, NFIDs::new(), resource_address.clone());

                return Some((resource_address, (vault_address, asset)));
            }

            None
        })
        .collect()
}

pub fn parse_non_fungibles_data_response_for_asset(
    mut asset: NonFungibleAsset,
    response: StateNonFungibleDataResponse,
) -> NonFungibleAsset {
    let nfids = response
        .non_fungible_ids
        .into_iter()
        .map(|non_fungible| {
            let id = non_fungible.non_fungible_id;
            let nft_data = non_fungible
                .data
                .and_then(|data| {
                    Some(
                        serde_json::from_value::<NFIdData>(data.programmatic_json.0)
                            .unwrap_or_else(|err| {
                                debug_println!(
                                    "{}{}:{}",
                                    debug_info!("Failed to parse non fungible data for id "),
                                    id,
                                    err
                                );
                                NFIdData { fields: Vec::new() }
                            }),
                    )
                })
                .unwrap();

            NFID::from_nfid_data(id, nft_data.fields)
        })
        .collect::<Vec<NFID>>()
        .into();

    asset.nfids = nfids;
    asset
}

async fn get_icon(icon_url: Option<String>, resource_address: &ResourceAddress) -> Option<Icon> {
    let url = icon_url?;
    let mut icon_path = AppPath::get().icons_directory();
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

#[cfg(test)]
mod tests {
    use crate::radix_dlt::gateway_requests;
    use radix_gateway_sdk::Network;

    use super::*;

    // #[tokio::test]
    // async fn test_update_assets_from_account_details_response() {
    //     let network = Network::Mainnet;
    //     let account_address = AccountAddress::from_str(
    //         "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
    //     )
    //     .unwrap();
    //     let mut response =
    //         gateway_requests::get_entities_details(network, &[account_address.to_string()])
    //             .await
    //             .unwrap();

    //     let mut fungible_assets: HashMap<ResourceAddress, FungibleAsset> = HashMap::new();
    //     let mut non_fungible_assets: HashMap<ResourceAddress, NonFungibleAsset> = HashMap::new();

    //     let new_assets = parse_account_details_response_and_update_assets(
    //         account_address,
    //         &mut fungible_assets,
    //         &mut non_fungible_assets,
    //         response.items.remove(0),
    //     );

    //     assert!(fungible_assets.len() != 0);
    //     assert!(non_fungible_assets.len() != 0);
    //     assert_eq!(new_assets.new_fungibles.len(), fungible_assets.len());
    //     assert_eq!(
    //         new_assets.new_non_fungibles.inner().len(),
    //         non_fungible_assets.len()
    //     );
    // }
}
