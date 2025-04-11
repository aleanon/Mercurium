use deps_two::*;

use std::future::IntoFuture;

use futures::future::join_all;
use radix_gateway_sdk::{
    generated::{
        model::{
            LedgerStateSelector, ResourceAggregationLevel, StateEntityDetailsOptIns,
            StateEntityDetailsResponse, StateEntityFungiblesPageResponse,
            StateEntityNonFungibleIdsPageResponse, StateEntityNonFungiblesPageResponse,
            StateNonFungibleDataResponse, StreamTransactionsResponse, TransactionDetailsOptIns,
        },
        request::StreamTransactionsRequired,
    },
    Client,
};
use types::{Network, UnsafeRef};

pub const ENTITY_DETAILS_MAX_ADDRESSES: usize = 20;

/// Takes a maximum of 20 addresses, otherwise it will panic
pub async fn get_entity_details(
    network: Network,
    addresses: &[&str],
) -> Result<StateEntityDetailsResponse, radix_gateway_sdk::Error> {
    assert!(addresses.len() <= ENTITY_DETAILS_MAX_ADDRESSES);

    let opt_ins = StateEntityDetailsOptIns {
        explicit_metadata: Some(vec!["symbol".to_string()]),
        ..Default::default()
    };

    Client::new(network.into(), None, None)?
        .get_inner_client()
        .state_entity_details(addresses, ResourceAggregationLevel::Vault, opt_ins)
        .into_future()
        .await
}

pub async fn get_fungible_balances_for_entity(
    network: Network,
    address: &str,
    cursor: Option<String>,
    at_state_version: Option<i64>,
) -> Result<StateEntityFungiblesPageResponse, radix_gateway_sdk::Error> {
    let client = Client::new(network.into(), None, None)?;
    let mut fluent_request = client.get_inner_client().entity_fungibles_page(
        address,
        ResourceAggregationLevel::Global,
        Default::default(),
    );

    if at_state_version.is_some() {
        fluent_request.params.at_ledger_state = Some(LedgerStateSelector {
            state_version: at_state_version,
            ..Default::default()
        });
        fluent_request.params.cursor = cursor;
    }

    fluent_request.into_future().await
}

/// If cursor is provided, a ledger state is also required, else the cursor will do nothing
pub async fn get_non_fungible_balances_for_entity(
    network: Network,
    address: &str,
    cursor: Option<String>,
    at_state_version: Option<i64>,
) -> Result<StateEntityNonFungiblesPageResponse, radix_gateway_sdk::Error> {
    let client = Client::new(network.into(), None, None)?;
    let mut fluent_request = client.get_inner_client().entity_non_fungibles_page(
        address,
        ResourceAggregationLevel::Vault,
        Default::default(),
    );

    if at_state_version.is_some() {
        fluent_request.params.at_ledger_state = Some(LedgerStateSelector {
            state_version: at_state_version,
            ..Default::default()
        });
        fluent_request.params.cursor = cursor;
    }

    fluent_request.into_future().await
}

pub async fn get_non_fungible_ids_from_vault(
    network: Network,
    account_address: &str,
    resource_address: &str,
    vault_address: &str,
    cursor: Option<String>,
    at_state_version: Option<i64>,
) -> Result<StateEntityNonFungibleIdsPageResponse, radix_gateway_sdk::Error> {
    let client = Client::new(network.into(), None, None)?;
    let mut fluent_request = client.get_inner_client().entity_non_fungible_ids_page(
        account_address,
        resource_address,
        vault_address,
    );

    if at_state_version.is_some() {
        fluent_request.params.at_ledger_state = Some(LedgerStateSelector {
            state_version: at_state_version,
            ..Default::default()
        });
        fluent_request.params.cursor = cursor;
    }

    fluent_request.into_future().await
}

pub async fn get_non_fungible_data(
    network: Network,
    resource_address: &str,
    non_fungible_ids: &[String],
) -> Result<StateNonFungibleDataResponse, radix_gateway_sdk::Error> {
    let tasks = non_fungible_ids.chunks(100).map(|chunk| {
        let resource_address = unsafe{UnsafeRef::new(resource_address)};
        let chunk = chunk.to_owned();

        tokio::task::spawn(async move {
            let addresses: Vec<&str> = chunk.iter().map(|address| address.as_str()).collect();
            Client::new(network.into(), None, None)?
                .get_inner_client()
                .non_fungible_data(addresses.as_slice(), &*resource_address)
                .into_future()
                .await
        })
    });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(|join_result| join_result.ok())
        .fold(
            Err(radix_gateway_sdk::Error::NetworkInvalid),
            |acc, response_result| match response_result {
                Ok(response) => match acc {
                    Ok(mut ok_acc) => {
                        ok_acc.non_fungible_ids.extend(response.non_fungible_ids);
                        Ok(ok_acc)
                    }
                    Err(_) => Ok(response),
                },
                Err(e) => acc.map_err(|_| e),
            },
        )
}

pub async fn get_transactions_for_entity_from_ledger_state_version(
    network: Network,
    address: String,
    cursor: Option<String>,
    at_state_version: Option<i64>,
) -> Result<StreamTransactionsResponse, radix_gateway_sdk::Error> {
    let opt_ins = TransactionDetailsOptIns {
        balance_changes: Some(true),
        ..Default::default()
    };

    let stream_transactions_required = StreamTransactionsRequired {
        affected_global_entities_filter: &[address.as_str()],
        opt_ins,
        order: "Desc",
        accounts_with_manifest_owner_method_calls: &[],
        accounts_without_manifest_owner_method_calls: &[],
        events_filter: Vec::new(),
        kind_filter: "",
        manifest_accounts_deposited_into_filter: &[],
        manifest_accounts_withdrawn_from_filter: &[],
        manifest_badges_presented_filter: &[],
        manifest_class_filter: serde_json::Value::Null,
        manifest_resources_filter: &[],
    };
    let client = Client::new(network.into(), None, None)?;
    let mut fluent_request = client
        .get_inner_client()
        .stream_transactions(stream_transactions_required);

    if at_state_version.is_some() {
        fluent_request.params.at_ledger_state = Some(LedgerStateSelector {
            state_version: at_state_version,
            ..Default::default()
        });
        fluent_request.params.cursor = cursor;
    }

    fluent_request.into_future().await
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use types::{
        address::{AccountAddress, Address, ResourceAddress},
        Network,
    };

    use super::*;

    #[tokio::test]
    async fn test_get_entities_details() {
        let account_address = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .unwrap();

        let network = Network::Mainnet;
        let account_details = get_entity_details(network, &[account_address.as_str()])
            .await
            .unwrap();

        assert_eq!(account_details.items.len(), 1);
        assert!(account_details.items[0].fungible_resources.is_some());
        assert!(account_details.items[0].non_fungible_resources.is_some());

        let resource_address = ResourceAddress::from_str(
            "resource_rdx1t4h4396mukhpzdrr5sfvegjsxl8q7a34q2vkt4quxcxahna8fucuz4",
        )
        .unwrap();
        let resource_details = get_entity_details(network, &[resource_address.as_str()])
            .await
            .unwrap();

        assert_eq!(resource_details.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_fungible_balances() {
        let network = Network::Mainnet;

        let account_address = AccountAddress::from_str(
            "account_rdx16y60m8p2lxl72rdqcxh6wj270ckku7e3hrr6fra05f9p34zlqwgd0k",
        )
        .unwrap();

        let response = match get_fungible_balances_for_entity(
            network.into(),
            account_address.as_str(),
            None,
            None,
        )
        .await
        {
            Ok(response) => response,
            Err(err) => panic!("{err}"),
        };

        assert_eq!(response.items.len(), 100);

        let ledger_state = Some(response.ledger_state_mixin.ledger_state.state_version);
        let cursor = response.next_cursor.clone();

        if cursor.is_some() {
            let response = match get_fungible_balances_for_entity(
                network.into(),
                account_address.as_str(),
                cursor,
                ledger_state,
            )
            .await
            {
                Ok(response) => response,
                Err(err) => panic!("{err}"),
            };

            assert_eq!(response.items.len(), 100);
        }
    }
}
