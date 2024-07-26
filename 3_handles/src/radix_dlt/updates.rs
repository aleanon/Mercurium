use super::*;
use debug_print::debug_println;
use iced::futures::future::join_all;
use std::{collections::HashMap, sync::Arc};
use store::AsyncDb;
use thiserror::Error;
use tokio::task::JoinHandle;
use types::{
    address::{AccountAddress, Address, ResourceAddress},
    assets::NFID,
    assets::{FungibleAsset, NonFungibleAsset},
    collections::{AccountUpdate, AccountsUpdate},
    Account, Network, Resource, Ur,
};

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Error connecting to gateway")]
    GatewayError(#[from] radix_gateway_sdk::Error),
    #[error("Error parsing response")]
    ResponseParseError,
    #[error("Error pasing address")]
    AddressParseError,
    #[error("No assets found")]
    EmptyResponse,
}

pub async fn update_all_accounts(network: Network, db: &AsyncDb) -> AccountsUpdate {
    let accounts = db.get_accounts().await.unwrap_or(Vec::new());
    let resource_map = db.get_all_resources().await.unwrap_or(HashMap::new());
    let resources = Arc::new(resource_map);

    update_accounts(network, resources, accounts).await
}

pub async fn update_accounts(
    network: Network,
    resources: Arc<HashMap<ResourceAddress, Resource>>,
    accounts: Vec<Account>,
) -> AccountsUpdate {
    // `resources` is inside an Arc to make sure it is valid for the duration of this task
    // From this point we know that the resources will be valid until all tasks within this function are finished,
    // therefore we pass around a non reference counted unsafe reference to resources to sub tasks
    let resources = unsafe { Ur::new(&*resources) };

    let tasks = accounts.into_iter().map(|account| {
        let resources = resources.clone();
        tokio::spawn(async move { update_account(network, resources, account).await })
    });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(|join_result| {
            #[cfg(debug_assertions)]
            if let Err(err) = &join_result {
                println!("Failed to join task {}", err);
            }

            join_result.ok()
        })
        .fold(
            AccountsUpdate::new(network),
            |mut acc, (accountupdate, new_resources)| {
                new_resources
                    .into_iter()
                    .for_each(|(resource_address, (resource, url))| {
                        acc.new_resources.insert(resource_address.clone(), resource);
                        acc.icon_urls.insert(resource_address, url);
                    });
                acc.account_updates.push(accountupdate);
                acc
            },
        )
}

async fn update_account(
    network: Network,
    resources: Ur<HashMap<ResourceAddress, Resource>>,
    mut account: Account,
) -> (AccountUpdate, HashMap<ResourceAddress, (Resource, String)>) {
    let balances_last_updated_at_state_version = account.balances_last_updated.unwrap_or(0);
    let transactions_last_updated = account.transactions_last_updated;

    // The account address should be used througout tasks and is never mutated or removed from the Account struct.
    // by the end of this function all tasks will be completed, so the Ur will never be used while the reference is not valid
    let account_address_ur = unsafe { Ur::new(&account.address) };

    let account_address = account_address_ur.clone();
    let stored_resources = resources.clone();
    let fungible_assets_task = tokio::spawn(async move {
        update_fungible_assets_and_resources_for_account(
            network,
            account_address,
            stored_resources,
            balances_last_updated_at_state_version,
        )
        .await
    });

    let account_address = account_address_ur.clone();
    let stored_resources = resources.clone();
    let non_fungible_assets_task = tokio::spawn(async move {
        update_non_fungible_assets_and_resources_for_account(
            network,
            account_address,
            stored_resources,
            balances_last_updated_at_state_version,
        )
        .await
    });
    let (new_state_version_fungible_balances, fungible_assets, new_fungible_resources) =
        fungible_assets_task
            .await
            .and_then(|result| Ok(result.unwrap_or((0, HashMap::new(), HashMap::new()))))
            .unwrap_or_else(|err| {
                debug_println!("Failed to update fungible assets: {}", err);
                (0, HashMap::new(), HashMap::new())
            });

    let (new_state_version_non_fungible_balances, non_fungible_assets, new_non_fungible_resources) =
        non_fungible_assets_task
            .await
            .and_then(|result| Ok(result.unwrap_or((0, HashMap::new(), HashMap::new()))))
            .unwrap_or_else(|err| {
                debug_println!("Failed to get Non fungible assets {}", err);
                (0, HashMap::new(), HashMap::new())
            });

    let new_state_version = balances_last_updated_at_state_version
        .max(new_state_version_fungible_balances.min(new_state_version_non_fungible_balances));

    account.balances_last_updated = Some(new_state_version);
    let mut new_resources = new_fungible_resources;
    new_resources.extend(new_non_fungible_resources);

    (
        AccountUpdate {
            account,
            fungibles: fungible_assets,
            non_fungibles: non_fungible_assets,
        },
        new_resources,
    )
}

/// returns the updated `Resource` and the accompanying icon url
pub async fn update_resources(
    network: Network,
    resources: Vec<ResourceAddress>,
) -> HashMap<ResourceAddress, (Resource, String)> {
    const CHUNK_SIZE: usize = 20;
    
    let tasks = resources.chunks(CHUNK_SIZE).map(|chunk| {
        let chunk = chunk.to_owned();

        tokio::spawn(async move {
            let addresses = chunk
                .iter()
                .map(|address| address.as_str())
                .collect::<Vec<_>>();

            let response = gateway_requests::get_entity_details(network.into(), &addresses).await?;

            let new_resources = response
                .items
                .into_iter()
                .filter_map(|response_item| {
                    let (resource_address, resource_and_icon_url) =
                        parse_responses::parse_resource_details_response(response_item)?;
                    Some((resource_address, resource_and_icon_url))
                })
                .collect::<HashMap<ResourceAddress, (Resource, String)>>();

            Ok::<_, UpdateError>(new_resources)
        })
    });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(|join_result| join_result.ok()?.ok())
        .reduce(|mut acc, new_resources| {
            acc.extend(new_resources);
            acc
        })
        .unwrap_or(HashMap::new())
}

pub async fn update_fungible_assets_and_resources_for_account(
    network: Network,
    account_address: Ur<AccountAddress>,
    stored_resources: Ur<HashMap<ResourceAddress, Resource>>,
    last_updated_at_state_version: i64,
) -> Result<
    (
        i64,
        HashMap<ResourceAddress, FungibleAsset>,
        HashMap<ResourceAddress, (Resource, String)>,
    ),
    UpdateError,
> {
    let (state_version, assets) = update_fungible_balances_for_account(
        network,
        &account_address,
        last_updated_at_state_version,
    )
    .await?;

    let new_resource_addresses = assets
        .iter()
        .filter_map(|(resource_address, _)| {
            if !stored_resources.contains_key(resource_address) {
                Some(resource_address.clone())
            } else {
                None
            }
        })
        .collect::<Vec<ResourceAddress>>();

    let new_resources = update_resources(network.into(), new_resource_addresses).await;

    Ok::<_, UpdateError>((state_version, assets, new_resources))
}

pub async fn update_fungible_balances_for_account(
    network: Network,
    account_address: &AccountAddress,
    last_updated_at_state_version: i64,
) -> Result<(i64, HashMap<ResourceAddress, FungibleAsset>), UpdateError> {
    let mut response = gateway_requests::get_fungible_balances_for_entity(
        network,
        account_address.as_str(),
        None,
        None,
    )
    .await?;

    let mut result = (
        response.ledger_state_mixin.ledger_state.state_version,
        HashMap::new(),
    );

    'main: loop {
        let next_cursor = response.next_cursor.take();
        if next_cursor.is_none() {
            let parsed = parse_responses::parse_fungible_balances_response(
                response,
                last_updated_at_state_version,
                &account_address,
            );
            result.1.extend(parsed);
            break;
        } else {
            let address = account_address.clone();
            let cursor = next_cursor.clone();
            let at_state_version = Some(result.0);
            let next_response = tokio::spawn(async move {
                gateway_requests::get_fungible_balances_for_entity(
                    network,
                    address.as_str(),
                    cursor,
                    at_state_version,
                )
                .await
            });

            let parsed = parse_responses::parse_fungible_balances_response(
                response,
                last_updated_at_state_version,
                &account_address,
            );
            result.1.extend(parsed);

            match next_response.await {
                Ok(response_result) => match response_result {
                    Ok(new_response) => response = new_response,
                    Err(_) => {
                        for _ in 0..3 {
                            let retry_result = gateway_requests::get_fungible_balances_for_entity(
                                network,
                                account_address.as_str(),
                                next_cursor.clone(),
                                Some(result.0),
                            )
                            .await;
                            match retry_result {
                                Ok(new_response) => {
                                    response = new_response;
                                    continue 'main;
                                }
                                Err(_) => continue,
                            }
                        }
                        debug_println!(
                            "Failed to get fungible balances for {}",
                            account_address.as_str()
                        );
                        break;
                    }
                },
                Err(_) => {
                    debug_println!(
                        "Join error when getting fungible balances for {}",
                        account_address.as_str()
                    );
                    break;
                }
            }
        }
    }

    Ok(result)
}

pub async fn update_non_fungible_assets_and_resources_for_account(
    network: Network,
    account_address: Ur<AccountAddress>,
    stored_resources: Ur<HashMap<ResourceAddress, Resource>>,
    last_updated_at_state_version: i64,
) -> Result<
    (
        i64,
        HashMap<ResourceAddress, NonFungibleAsset>,
        HashMap<ResourceAddress, (Resource, String)>,
    ),
    UpdateError,
> {
    let (state_version, assets) = update_non_fungible_assets_for_account(
        network,
        &account_address,
        last_updated_at_state_version,
    )
    .await?;

    let assets_with_ids =
        update_non_fungible_ids_for_assets(network, &account_address, assets).await?;

    let assets_with_nfdata = update_non_fungible_data_for_ids(network, assets_with_ids).await;

    let new_resource_addresses = assets_with_nfdata
        .iter()
        .filter_map(|(resource_address, _)| {
            if !stored_resources.contains_key(resource_address) {
                Some(resource_address.clone())
            } else {
                None
            }
        })
        .collect::<Vec<ResourceAddress>>();

    let new_resources = update_resources(network, new_resource_addresses).await;

    Ok((state_version, assets_with_nfdata, new_resources))
}

pub async fn update_non_fungible_assets_for_account(
    network: Network,
    account_address: &AccountAddress,
    last_updated_at_state_version: i64,
) -> Result<(i64, HashMap<ResourceAddress, (String, NonFungibleAsset)>), UpdateError> {
    let mut response = gateway_requests::get_non_fungible_balances_for_entity(
        network,
        account_address.as_str(),
        None,
        None,
    )
    .await?;

    let mut result = (
        response.ledger_state_mixin.ledger_state.state_version,
        HashMap::new(),
    );

    'main: loop {
        let next_cursor = response
            .non_fungible_resources_collection
            .next_cursor
            .take();

        if next_cursor.is_none() {
            let parsed = parse_responses::parse_non_fungible_balances_response_without_nfids(
                response,
                last_updated_at_state_version,
                &account_address,
            );
            result.1.extend(parsed);
            break;
        } else {
            let address = account_address.clone();
            let cursor = next_cursor.clone();
            let at_state_version = Some(result.0);
            let next_response = tokio::spawn(async move {
                gateway_requests::get_non_fungible_balances_for_entity(
                    network,
                    address.as_str(),
                    cursor,
                    at_state_version,
                )
                .await
            });

            let parsed = parse_responses::parse_non_fungible_balances_response_without_nfids(
                response,
                last_updated_at_state_version,
                &account_address,
            );
            result.1.extend(parsed);

            match next_response.await {
                Ok(response_result) => match response_result {
                    Ok(new_response) => response = new_response,
                    Err(_) => {
                        for _ in 0..3 {
                            let retry_result =
                                gateway_requests::get_non_fungible_balances_for_entity(
                                    network,
                                    account_address.as_str(),
                                    next_cursor.clone(),
                                    Some(result.0),
                                )
                                .await;
                            match retry_result {
                                Ok(new_response) => {
                                    response = new_response;
                                    continue 'main;
                                }
                                Err(_) => continue,
                            }
                        }
                        debug_println!(
                            "Failed to get non-fungible balances for account: {}",
                            account_address.as_str()
                        );
                        break;
                    }
                },
                Err(_) => {
                    debug_println!(
                        "Join Error when getting non_fungible balances for account: {}",
                        account_address.as_str()
                    );
                    break;
                }
            }
        }
    }
    Ok(result)
}

pub async fn update_non_fungible_ids_for_assets(
    network: Network,
    account_address: &AccountAddress,
    assets: HashMap<ResourceAddress, (String, NonFungibleAsset)>,
) -> Result<HashMap<ResourceAddress, NonFungibleAsset>, UpdateError> {
    let tasks = assets
        .into_iter()
        .map(|(resource_address, (vault_address, asset))| {
            let account_address = account_address.clone();
            tokio::spawn(async move {
                update_non_fungible_ids_for_asset(
                    network,
                    account_address,
                    resource_address,
                    vault_address,
                    asset,
                )
                .await
            })
        });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(|join_result| {
            #[cfg(debug_assertions)]
            if let Err(err) = &join_result {
                println!("Join Error when getting nfids {}", err)
            }
            join_result.ok()
        })
        .fold(Err(UpdateError::EmptyResponse), |mut acc, result| {
            match result {
                Ok((resource_address, asset)) => match acc {
                    Ok(ref mut map) => {
                        map.insert(resource_address, asset);
                    }
                    Err(_) => {
                        let mut map = HashMap::new();
                        map.insert(resource_address, asset);
                        acc = Ok(map);
                    }
                },
                Err(err) => {
                    debug_println!("Error when updating nfids {}", err);
                    acc = acc.map_err(|_| err)
                }
            }
            acc
        })
}
async fn update_non_fungible_ids_for_asset(
    network: Network,
    account_address: AccountAddress,
    resource_address: ResourceAddress,
    vault_address: String,
    mut asset: NonFungibleAsset,
) -> Result<(ResourceAddress, NonFungibleAsset), UpdateError> {
    let mut response = gateway_requests::get_non_fungible_ids_from_vault(
        network,
        account_address.as_str(),
        resource_address.as_str(),
        vault_address.as_str(),
        None,
        None,
    )
    .await?;

    let ledger_state_version = response.ledger_state.state_version;

    'main: loop {
        let next_cursor = response.non_fungible_ids_collection.next_cursor.take();

        if next_cursor.is_none() {
            asset.nfids.extend(
                response
                    .non_fungible_ids_collection
                    .items
                    .into_iter()
                    .map(|id| NFID::new(id)),
            );
            break;
        } else {
            let account_address_string = response.address;
            let resource_address_string = response.resource_address;
            let vault_address_clone = vault_address.clone();
            let cursor = next_cursor.clone();
            let next_response = tokio::spawn(async move {
                gateway_requests::get_non_fungible_ids_from_vault(
                    network,
                    account_address_string.as_str(),
                    resource_address_string.as_str(),
                    vault_address_clone.as_str(),
                    cursor,
                    Some(ledger_state_version),
                )
                .await
            });

            asset.nfids.extend(
                response
                    .non_fungible_ids_collection
                    .items
                    .into_iter()
                    .map(|id| NFID::new(id)),
            );

            match next_response.await {
                Ok(response_result) => match response_result {
                    Ok(new_response) => response = new_response,
                    Err(_) => {
                        for _ in 0..3 {
                            let retry_result = gateway_requests::get_non_fungible_ids_from_vault(
                                network,
                                account_address.as_str(),
                                resource_address.as_str(),
                                vault_address.as_str(),
                                next_cursor.clone(),
                                Some(ledger_state_version),
                            )
                            .await;
                            match retry_result {
                                Ok(new_response) => {
                                    response = new_response;
                                    continue 'main;
                                }
                                Err(_) => continue,
                            }
                        }
                        debug_println!(
                            "Failed to get nfids for account {}, resource {}",
                            account_address.as_str(),
                            resource_address.as_str()
                        );
                        break;
                    }
                },
                Err(_) => {
                    debug_println!(
                        "Join Error when getting nfids for account {}, resource {}",
                        account_address.as_str(),
                        resource_address.as_str()
                    );
                    break;
                }
            }
        }
    }

    Ok((resource_address, asset))
}

async fn update_non_fungible_data_for_ids(
    network: Network,
    assets: HashMap<ResourceAddress, NonFungibleAsset>,
) -> HashMap<ResourceAddress, NonFungibleAsset> {
    let mut tasks: Vec<JoinHandle<Result<(ResourceAddress, NonFungibleAsset), UpdateError>>> =
        Vec::new();

    assets
        .into_iter()
        .for_each(|(resource_address, mut asset)| {
            let ids = asset.nfids_as_string();
            for chunk in ids.chunks(100) {
                let chunk = chunk.to_vec();
                let resource_address = resource_address.clone();
                let asset = asset.clone();
                tasks.push(tokio::spawn(async move {
                    let response = gateway_requests::get_non_fungible_data(
                        network,
                        resource_address.as_str(),
                        chunk.as_slice(),
                    )
                    .await?;
                    let asset_with_nfdata =
                        parse_responses::parse_non_fungibles_data_response_for_asset(
                            asset, response,
                        );
                    Ok((resource_address, asset_with_nfdata))
                }))
            }
        });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(|join_result| {
            #[cfg(debug_assertions)]
            match &join_result {
                Ok(result) => {
                    if let Err(err) = &result {
                        println!("Failed to update non fungible data {}", err);
                    }
                }
                Err(err) => {
                    println!("Join error when updating non fungible data {}", err);
                }
            }
            join_result.ok()?.ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use types::Ed25519PublicKey;

    #[tokio::test]
    async fn test_update_accounts() {
        let network = Network::Mainnet;

        let account_address = AccountAddress::from_str(
            "account_rdx16y60m8p2lxl72rdqcxh6wj270ckku7e3hrr6fra05f9p34zlqwgd0k",
        )
        .unwrap();
        let account = Account::new(
            1,
            "test".to_string(),
            types::Network::Mainnet,
            [0; 6],
            account_address,
            Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
        );
        let mut updated_accounts_entities =
            update_accounts(network, Arc::new(HashMap::new()), vec![account]).await;

        let account_entities = updated_accounts_entities.account_updates.remove(0);
        let new_fungibles = account_entities.fungibles;
        let new_non_fungibles = account_entities.non_fungibles;
        let new_resources = updated_accounts_entities.new_resources;

        assert!(new_fungibles.len() != 0);
        assert_eq!(
            new_resources.len(),
            new_fungibles.len() + new_non_fungibles.len()
        );
    }
}
