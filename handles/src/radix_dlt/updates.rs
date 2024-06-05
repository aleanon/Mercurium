use super::*;
use debug_print::debug_println;
use iced::futures::future::join_all;
use parse_responses::parse_resource_details_response;
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
    sync::Arc,
};
use store::Db;
use thiserror::Error;
use types::{
    assets::{FungibleAsset, NewAssets, NonFungibleAsset},
    debug_info, AccountAddress, Resource, ResourceAddress, Transaction,
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
    NoAssetsFound,
}

pub struct AccountUpdateEntities {
    pub account_address: AccountAddress,
    pub fungibles: HashMap<ResourceAddress, FungibleAsset>,
    pub non_fungibles: HashMap<ResourceAddress, NonFungibleAsset>,
}

pub struct UpdateAccountsResult {
    pub accounts_entities: Vec<AccountUpdateEntities>,
    pub new_resources: HashMap<ResourceAddress, Resource>,
    pub icon_urls: BTreeMap<ResourceAddress, String>,
}

impl UpdateAccountsResult {
    pub fn new() -> Self {
        Self {
            accounts_entities: Vec::new(),
            new_resources: HashMap::new(),
            icon_urls: BTreeMap::new(),
        }
    }
}

pub async fn update_all_accounts(
    gateway_client: Arc<radix_gateway_sdk::Client>,
    db: Db,
) -> Result<UpdateAccountsResult, UpdateError> {
    let account_addresses = db.get_account_addresses().unwrap_or(Vec::new());

    update_accounts(account_addresses, gateway_client, db).await
}

pub async fn update_accounts(
    account_addresses: Vec<AccountAddress>,
    gateway_client: Arc<radix_gateway_sdk::Client>,
    db: Db,
) -> Result<UpdateAccountsResult, UpdateError> {
    let resources = Arc::new(db.get_all_resources().unwrap_or(HashMap::new()));
    let mut fungible_assets = db
        .get_fungible_assets_for_accounts(account_addresses.as_slice())
        .unwrap_or(HashMap::new());
    let mut non_fungible_assets = db
        .get_non_fungible_assets_for_accounts(account_addresses.as_slice())
        .unwrap_or(HashMap::new());

    let tasks = account_addresses.into_iter().map(|account_address| {
        let client = gateway_client.clone();
        let resources = resources.clone();
        let fungible_assets_for_account = fungible_assets
            .remove(&account_address)
            .unwrap_or(HashMap::new());
        let non_fungible_assets_for_account = non_fungible_assets
            .remove(&account_address)
            .unwrap_or(HashMap::new());

        let account_update_entities = AccountUpdateEntities {
            account_address,
            fungibles: fungible_assets_for_account,
            non_fungibles: non_fungible_assets_for_account,
        };

        tokio::spawn(
            async move { update_account(client, resources, account_update_entities).await },
        )
    });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(|join_result| join_result.ok())
        .fold::<Result<
            (
                Vec<AccountUpdateEntities>,
                HashMap<ResourceAddress, Resource>,
                BTreeMap<ResourceAddress, String>,
            ),
            UpdateError,
        >, _>(Err(UpdateError::NoAssetsFound), |mut acc, response| {
            match response {
                Ok(ok_response) => match acc {
                    Ok((ref mut account_entities, ref mut new_resources, ref mut icon_urls)) => {
                        account_entities.push(ok_response.0);
                        new_resources.extend(ok_response.1.into_iter());
                        icon_urls.extend(ok_response.2.into_iter());
                    }
                    Err(_) => acc = Ok((vec![ok_response.0], ok_response.1, ok_response.2)),
                },
                Err(err) => acc = acc.map_err(|_| err),
            }
            acc
        })
        .and_then(|(accounts_entities, new_resources, icon_urls)| {
            Ok(UpdateAccountsResult {
                accounts_entities,
                new_resources,
                icon_urls,
            })
        })
}

pub async fn update_account(
    gateway_client: Arc<radix_gateway_sdk::Client>,
    resources: Arc<HashMap<ResourceAddress, Resource>>,
    mut account_update_entities: AccountUpdateEntities,
) -> Result<
    (
        AccountUpdateEntities,
        HashMap<ResourceAddress, Resource>,
        BTreeMap<ResourceAddress, String>,
    ),
    UpdateError,
> {
    // Account details include fungible and non-fungible assets held in the account
    let mut account_details_response = gateway_requests::get_entities_details(
        gateway_client.clone(),
        &[account_update_entities.account_address.to_string()],
    )
    .await?;

    // Parses the above response and updated all fungibles and non fungibles already held in the account
    // any new assets are returned for further processing
    let mut new_assets = NewAssets::new();
    if account_details_response.items.len() > 0 {
        let entity_response = account_details_response.items.remove(0);
        let account_address = AccountAddress::from_str(entity_response.address.as_str())
            .map_err(|_| UpdateError::AddressParseError)?;

        new_assets = parse_responses::parse_account_details_response_and_update_assets(
            account_address,
            &mut account_update_entities.fungibles,
            &mut account_update_entities.non_fungibles,
            entity_response,
        )
    }

    debug_println!("Parsed {} new fungibles", new_assets.new_fungibles.len());

    // Collects all fungible and non fungible new resources in one collection
    let mut new_resource_addresses = new_assets.new_fungibles;

    let new_non_fungibles = new_assets.new_non_fungibles.inner();

    for (resource_address, _) in &new_non_fungibles {
        new_resource_addresses.insert(resource_address.clone());
    }

    debug_println!("Found {} new resources", new_resource_addresses.len());

    // Sorts out the resources we already have in our database, this includes resources held in other accounts
    let new_resource_addresses_as_string = new_resource_addresses
        .iter()
        .filter_map(|resource_address| {
            if !resources.contains_key(resource_address) {
                Some(resource_address.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    debug_println!(
        "Number of new resources as string: {}",
        new_resource_addresses_as_string.len()
    );

    // Gets the details of all resources not held in any accounts
    let resources_detail_response = gateway_requests::get_entities_details(
        gateway_client.clone(),
        new_resource_addresses_as_string.as_slice(),
    )
    .await?;

    debug_println!("Successfully retrieved details for new resources");

    // Parses resource details and gets icon urls for new resources
    let mut new_resources = HashMap::with_capacity(resources_detail_response.items.len());
    let mut icon_urls = BTreeMap::new();

    for item in resources_detail_response.items {
        if let Some(((resource_address, resource), url)) = parse_resource_details_response(item) {
            new_resources.insert(resource_address, resource);

            if let Some((resource_address, url)) = url {
                icon_urls.insert(resource_address, url);
            }
        }
    }

    debug_println!(
        "Successfully parsed {} resource details",
        new_resources.len()
    );

    // Send a request for details on all new NFT ids, creates a task for each resource,
    // the result of the taks are collected and stored in the return value
    let non_fungibles_data = {
        let tasks = new_non_fungibles
            .into_iter()
            .map(|(resource_address, ids)| {
                let client = gateway_client.clone();
                let resource_address = resource_address;
                tokio::task::spawn(async move {
                    gateway_requests::get_non_fungible_data(
                        client,
                        resource_address.as_str(),
                        ids.as_slice(),
                    )
                    .await
                })
            });

        join_all(tasks)
            .await
            .into_iter()
            .filter_map(|join_result| {
                let response = join_result.ok()?.ok()?;
                let resource_address =
                    ResourceAddress::from_str(response.resource_address.as_str()).ok()?;

                let (resource_address, asset) = account_update_entities
                    .non_fungibles
                    .remove_entry(&resource_address)?;
                let asset =
                    parse_responses::parse_non_fungibles_data_response_for_asset(asset, response);
                Some((resource_address, asset))
            })
            .collect::<HashMap<ResourceAddress, NonFungibleAsset>>()
    };

    debug_println!(
        "Successfully parsed {} non fungible ids",
        non_fungibles_data.len()
    );

    account_update_entities
        .non_fungibles
        .extend(non_fungibles_data.into_iter());

    Ok((account_update_entities, new_resources, icon_urls))
}

pub async fn update_transactions_for_account(
    gateway_client: Arc<radix_gateway_sdk::Client>,
    db: Db,
    account_address: AccountAddress,
) -> Result<BTreeMap<Transaction>, UpdateError> {
    let last_updated = 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_accounts() {
        let gateway_client = Arc::new(
            radix_gateway_sdk::Client::new(radix_gateway_sdk::Network::Mainnet, None, None)
                .unwrap(),
        );

        let db = Db::new_in_memory();

        let account_address = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .unwrap();

        let mut updated_accounts_entities =
            update_accounts(vec![account_address], gateway_client, db)
                .await
                .unwrap();

        let account_entities = updated_accounts_entities.accounts_entities.remove(0);
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
