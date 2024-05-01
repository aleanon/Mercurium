use super::*;
use std::sync::Arc;
use types::{radix_request_client::RadixDltRequestClient, Account, EntityAccount, Network};

pub async fn update_account(
    client: RadixDltRequestClient,
    account: Account,
    network: Network,
) -> Result<EntityAccount, HandleError> {
    let mut entity_details_response =
        gateway_requests::get_entity_details(&client, &[account.address.as_str()], network).await?;

    if entity_details_response.items.len() > 0 {
        update_account_data(&client, entity_details_response.items.remove(0), account).await
    } else {
        Err(HandleError::NoDetailsFound)
    }
}

pub async fn update_accounts(
    client: RadixDltRequestClient,
    accounts: Vec<Account>,
    network: Network,
) -> Result<Vec<EntityAccount>, HandleError> {
    let addresses = accounts
        .iter()
        .map(|account| account.address.as_str())
        .collect::<Vec<&str>>();

    // Send a request to the Radix gateway for the details of these accounts
    let accounts_response =
        gateway_requests::get_entity_details(&client, addresses.as_slice(), network)
            .await?
            .items;

    // Create a task for each account that will get the details of each asset in the account
    let tasks = accounts_response.into_iter().map(|entity_account| {
        let coms = client.clone();
        let account = accounts
            .iter()
            .enumerate()
            .find(|(_, account)| account.address.as_str() == entity_account.address.as_str());

        let account = match account {
            Some((i, _)) => Some(accounts.remove(i)),
            None => None,
        };

        tokio::spawn(async move {
            match account {
                Some(account) => Self::update_account_data(coms, entity_account, account).await,
                None => Err(HandleError::EntityNotFound(entity_account.address)),
            }
        })
    });

    let accounts = join_all(tasks)
        .await
        .into_iter()
        .filter_map(|result| {
            #[cfg(debug_assertions)]
            match &result {
                Ok(value) => match value {
                    Ok(account) => println!(
                        "Successfully retrieved data for account {}",
                        account.address.as_str()
                    ),
                    Err(err) => println!("Failed to retrieve account data, Error: {err}"),
                },
                Err(err) => println!("Failed to retrieve account data, Error: {err}"),
            }

            result.ok().and_then(|result| result.ok())
        })
        .collect();

    Ok(accounts)
}

pub async fn update_account_data(
    coms: RadixDltRequestClient,
    account_response: Entity,
    mut account: Account,
) -> Result<EntityAccount, HandleError> {
    let fungible_resources = account_response
        .fungible_resources
        .items
        .into_iter()
        .map(|fungible| (fungible.resource_address.to_owned(), fungible))
        .collect::<HashMap<String, FungibleResource>>();
    let fungible_resources = Arc::new(fungible_resources);

    let non_fungible_resources = account_response
        .non_fungible_resources
        .items
        .into_iter()
        .map(|non_fungible| (non_fungible.resource_address.to_owned(), non_fungible))
        .collect::<HashMap<String, NonFungibleResource>>();
    let non_fungible_resources = Arc::new(non_fungible_resources);

    let coms_clone = coms.clone();

    let fungibles_response =
        tokio::spawn(async move { Self::get_fungibles(coms_clone, fungible_resources).await });

    let non_fungibles_response =
        tokio::spawn(
            async move { Self::get_non_fungibles_details(coms, non_fungible_resources).await },
        );

    let (fungibles, non_fungibles) = join(fungibles_response, non_fungibles_response).await;
    let fungibles = fungibles??;
    let non_fungibles = non_fungibles??;

    account.fungibles = fungibles;
    account.non_fungibles = non_fungibles;
    Ok::<_, HandleError>(account)
}

async fn get_fungibles(
    coms: Arc<Coms>,
    fungible_resources: Arc<HashMap<String, FungibleResource>>,
) -> Result<Fungibles, HandleError> {
    let fungible_addresses = fungible_resources
        .keys()
        .map(|key| key.as_str())
        .collect::<Vec<_>>();

    let fungibles_details = coms
        .radixdlt_request_builder
        .get_entity_details(fungible_addresses.as_slice())
        .await?;

    let fungible_tasks = fungibles_details.items.into_iter().map(|fungible| {
        let fungible_resources = fungible_resources.clone();
        tokio::spawn(
            async move { Self::parse_fungible_response(fungible_resources, fungible).await },
        )
    });

    let joined = join_all(fungible_tasks)
        .await
        .into_iter()
        .filter_map(|result| result.ok())
        .collect::<Vec<_>>();

    let fungibles: Fungibles = joined
        .into_iter()
        .filter_map(|result| result.ok())
        .collect::<BTreeSet<_>>()
        .into();

    Ok::<_, HandleError>(fungibles)
}

async fn parse_fungible_response(
    fungible_resources: Arc<HashMap<String, FungibleResource>>,
    fungible: Entity,
) -> Result<Fungible, HandleError> {
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

    let address = ResourceAddress::from_str(&fungible.address)?;

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

    let icon = Self::get_icon(icon_url, &address).await;

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
    Ok::<_, HandleError>(fungible)
}

async fn get_non_fungibles_details(
    coms: Arc<Coms>,
    non_fungible_resources: Arc<HashMap<String, NonFungibleResource>>,
) -> Result<NonFungibles, HandleError> {
    let non_fungible_addresses = non_fungible_resources
        .keys()
        .map(|key| key.as_str())
        .collect::<Vec<&str>>();

    let non_fungibles_details = coms
        .radixdlt_request_builder
        .get_entity_details(non_fungible_addresses.as_slice())
        .await?;

    let tasks = non_fungibles_details.items.into_iter().map(|non_fungible| {
        let non_fungible_resources = non_fungible_resources.clone();
        tokio::spawn(async move {
            Self::non_fungible_response(non_fungible_resources, non_fungible).await
        })
    });

    let non_fungibles: NonFungibles = join_all(tasks)
        .await
        .into_iter()
        .filter_map(|result| result.ok().and_then(|result| result.ok()))
        .collect::<NonFungibles>()
        .into();

    Ok::<_, HandleError>(non_fungibles)
}
