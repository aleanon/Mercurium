use deps_two::*;

use std::collections::HashMap;
use std::str::FromStr;

use debug_print::debug_println;
use radix_gateway_sdk::generated::model::{
    StateEntityDetailsResponseItem, StateEntityFungiblesPageResponse,
    StateEntityNonFungiblesPageResponse, StateNonFungibleDataResponse,
};

use types::address::{AccountAddress, ResourceAddress};
use types::assets::{FungibleAsset, NFIDs, NonFungibleAsset, NFID};
use types::response_models::non_fungible_id_data::NFIdData;
use types::response_models::{
    FungibleCollectionItemGlobal, MetaDataStringArrayValue, MetaDataStringValue,
    NonFungibleCollectionItemVaultAggregated, ResourceDetails,
};
use types::{debug_info, Resource};

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

/// Returns a `HashMap` with a tuple value `(String, NonFungibleAsset)`, the `String` represents the vault address
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

#[cfg(test)]
mod tests {

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
