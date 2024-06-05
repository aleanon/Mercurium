use std::collections::{BTreeSet, HashMap};
use std::io::{BufWriter, Cursor};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;

use bytes::Bytes;
use debug_print::debug_println;
use image::imageops::FilterType;
use radix_gateway_sdk::generated::model::{
    FungibleResourcesCollection, NonFungibleResourcesCollection, StateEntityDetailsResponseItem,
    StateNonFungibleDataResponse,
};
use types::assets::{FungibleAsset, NewAssets, NonFungibleAsset};
use types::response_models::accounts_details::NonFungibleResourceVaultAggregated;
use types::response_models::non_fungible_id_data::NFIdData;
use types::response_models::{
    FungibleResourceVaultAggregated, MetaDataStringArrayValue, MetaDataStringValue, ResourceDetails,
};
use types::Resource;
use types::{AccountAddress, AppPath, Icon, NFIDs, RadixDecimal, ResourceAddress, NFID};

use crate::filesystem::resize_image::resize_image;

// pub fn parse_transactions_response(response: TransactionsResponse) -> Result<Transaction> {}

pub struct ParseResponseError;

pub fn parse_account_details_response_and_update_assets(
    account_address: AccountAddress,
    fungible_assets: &mut HashMap<ResourceAddress, FungibleAsset>,
    non_fungible_assets: &mut HashMap<ResourceAddress, NonFungibleAsset>,
    entity_response: StateEntityDetailsResponseItem,
) -> NewAssets {
    let mut new_assets = NewAssets::new();

    if let Some(fungible_resources_response) = entity_response.fungible_resources {
        parse_account_details_response_and_update_fungible_assets(
            &mut new_assets,
            &account_address,
            fungible_assets,
            fungible_resources_response,
        )
    };

    if let Some(non_fungible_resources_response) = entity_response.non_fungible_resources {
        parse_account_details_response_and_update_non_fungible_assets(
            &mut new_assets,
            &account_address,
            non_fungible_assets,
            non_fungible_resources_response,
        )
    }

    new_assets
}

fn parse_account_details_response_and_update_fungible_assets(
    new_assets: &mut NewAssets,
    account_address: &AccountAddress,
    assets: &mut HashMap<ResourceAddress, FungibleAsset>,
    fungible_resources_response: FungibleResourcesCollection,
) {
    for fungible_resource in fungible_resources_response.items {
        if let Ok(fungible_resource) =
            serde_json::from_value::<FungibleResourceVaultAggregated>(fungible_resource.0)
        {
            let resource_address =
                match ResourceAddress::from_str(&fungible_resource.resource_address) {
                    Ok(address) => address,
                    Err(_) => continue,
                };
            debug_println!("{}", resource_address.to_string());

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
                debug_println!("amount: {}", &amount);

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

fn parse_account_details_response_and_update_non_fungible_assets(
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

pub fn parse_resource_details_response(
    resource: StateEntityDetailsResponseItem,
) -> Option<(
    (ResourceAddress, Resource),
    Option<(ResourceAddress, String)>,
)> {
    let resource_address = ResourceAddress::from_str(&resource.address).ok()?;

    let (current_supply, divisibility) = resource
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
    let mut icon_url: Option<(ResourceAddress, String)> = None;
    let mut tags: Vec<String> = Vec::new();

    for metadataitem in resource.metadata.items {
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
                    icon_url = Some((resource_address.clone(), value.value));
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

    Some(((resource_address, resource), icon_url))
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
                .and_then(|data| serde_json::from_value::<NFIdData>(data.programmatic_json.0).ok())
                .unwrap_or(NFIdData { fields: Vec::new() });

            NFID::new_with_data(id, nft_data.fields)
        })
        .collect::<BTreeSet<NFID>>()
        .into();

    asset.nfids = nfids;
    asset
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

#[cfg(test)]
mod tests {
    use crate::radix_dlt::gateway_requests;
    use radix_gateway_sdk::Network;
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn test_update_assets_from_account_details_response() {
        let client =
            Arc::new(radix_gateway_sdk::Client::new(Network::Mainnet, None, None).unwrap());
        let account_address = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .unwrap();
        let mut response =
            gateway_requests::get_entities_details(client, &[account_address.to_string()])
                .await
                .unwrap();

        let mut fungible_assets: HashMap<ResourceAddress, FungibleAsset> = HashMap::new();
        let mut non_fungible_assets: HashMap<ResourceAddress, NonFungibleAsset> = HashMap::new();

        let new_assets = parse_account_details_response_and_update_assets(
            account_address,
            &mut fungible_assets,
            &mut non_fungible_assets,
            response.items.remove(0),
        );

        assert!(fungible_assets.len() != 0);
        assert!(non_fungible_assets.len() != 0);
        assert_eq!(new_assets.new_fungibles.len(), fungible_assets.len());
        assert_eq!(
            new_assets.new_non_fungibles.inner().len(),
            non_fungible_assets.len()
        );
    }
}
