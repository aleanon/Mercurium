use std::{error::Error, future::IntoFuture, io::Read};

use debug_print::debug_println;
use flate2::bufread::GzDecoder;
use radix_gateway_sdk::generated::model::{
    LedgerStateSelector, ResourceAggregationLevel, StateEntityDetailsOptIns,
    StateEntityDetailsResponse,
};
use serde_json::json;
use types::{
    radix_request_client::{RadixDltRequestClient, RadixDltRequestError},
    response_models::{
        accounts_details::AccountsDetails, AllFungiblesResponse, AllNFTsResponse,
        FungibleVaultsResponse, TransactionsResponse,
    },
    AccountAddress, Network, ResourceAddress,
};

use crate::radix_dlt::urls::{GET_ENTITY_DETAILS_MAINNET, GET_ENTITY_DETAILS_STOKENET};

use super::urls::{
    GET_COMPONENT_FUNGIBLES_MAINNET, GET_COMPONENT_FUNGIBLES_STOKENET, GET_COMPONENT_NFTS_MAINNET,
    GET_COMPONENT_NFTS_STOKENET, GET_FUNGIBLE_VAULTS_MAINNET, GET_FUNGIBLE_VAULTS_STOKENET,
    GET_TRANSACTIONS_STREAM_MAINNET, GET_TRANSACTIONS_STREAM_STOKENET,
};

// pub async fn get_all_fungibles_for_account(
//     client: &RadixDltRequestClient,
//     account: &AccountAddress,
//     network: Network,
// ) -> Result<AllFungiblesResponse, RadixDltRequestError> {
//     let url = match network {
//         Network::Mainnet => GET_COMPONENT_FUNGIBLES_MAINNET,
//         Network::Stokenet => GET_COMPONENT_FUNGIBLES_STOKENET,
//     };

//     let body = json!({
//         "address": account.as_str(),
//     });

//     let response = client.post(url).json(&body).send().await?;

//     let decoded = decode(response).await?;

//     let all_fungibles: AllFungiblesResponse = serde_json::from_str(&decoded)?;

//     Ok(all_fungibles)
// }

// pub async fn get_all_nfts_for_account(
//     client: &RadixDltRequestClient,
//     account: &AccountAddress,
//     network: Network,
// ) -> Result<AllNFTsResponse, RadixDltRequestError> {
//     let url = match network {
//         Network::Mainnet => GET_COMPONENT_NFTS_MAINNET,
//         Network::Stokenet => GET_COMPONENT_NFTS_STOKENET,
//     };

//     let body = json!({
//         "address": account.as_str()
//     });

//     let response = client.post(url).json(&body).send().await?;

//     let decoded = decode(response).await?;
//     let all_non_fungibles: AllNFTsResponse = serde_json::from_str(&decoded)?;

//     Ok(all_non_fungibles)
// }

// pub async fn get_fungible_vaults_for_account(
//     client: &RadixDltRequestClient,
//     account: &AccountAddress,
//     resource: ResourceAddress,
//     network: Network,
// ) -> Result<FungibleVaultsResponse, RadixDltRequestError> {
//     let url = match network {
//         Network::Mainnet => GET_FUNGIBLE_VAULTS_MAINNET,
//         Network::Stokenet => GET_FUNGIBLE_VAULTS_STOKENET,
//     };

//     let body = json!({
//         "address": account.as_str(),
//         "resource_address": resource.as_str(),
//     });

//     let response = client.post(url).json(&body).send().await?;

//     let decoded = decode(response).await?;
//     let fungible_vaults: FungibleVaultsResponse = serde_json::from_str(&decoded)?;

//     Ok(fungible_vaults)
// }

// pub async fn get_transactions_for_affected_entity(
//     client: &RadixDltRequestClient,
//     affected_entities_addresses: &[&str],
//     network: Network,
//     from_state_version: u64,
// ) -> Result<String, RadixDltRequestError> {
//     let url = match network {
//         Network::Mainnet => GET_TRANSACTIONS_STREAM_MAINNET,
//         Network::Stokenet => GET_TRANSACTIONS_STREAM_STOKENET,
//     };

//     let body = json!({
//         "limit_per_page": 100,
//         "affected_global_entities_filter": affected_entities_addresses,
//         "from_ledger_state": {
//             "state_version": from_state_version
//         },
//         "opt_ins": {
//         "balance_changes": true
//         }
//     });

//     let response = client.post(url).json(&body).send().await?;

//     let decoded = decode(response).await?;

//     #[cfg(test)]
//     {
//         use std::fs::File;
//         use std::io::Write;
//         let mut file = File::create("./transactions.json").unwrap();
//         let value: serde_json::Value = serde_json::from_str(&decoded).unwrap();
//         let formatted = serde_json::to_string_pretty(&value).unwrap();
//         file.write_all(&formatted.as_bytes()).unwrap();

//         Ok(formatted)
//     }

//     #[cfg(not(test))]
//     Ok(decoded)
// }

// pub async fn get_entity_details(
//     client: &RadixDltRequestClient,
//     addresses: &[&str],
//     network: Network,
// ) -> Result<String, RadixDltRequestError> {
//     let url = match network {
//         Network::Mainnet => GET_ENTITY_DETAILS_MAINNET,
//         Network::Stokenet => GET_ENTITY_DETAILS_STOKENET,
//     };

//     let body = json!({
//         "addresses": addresses,
//         "aggregation_level": "Vault",
//         "opt_ins": {
//             "ancestor_identities": true,
//             "component_royalty_vault_balance": true,
//             "package_royalty_vault_balance": true,
//             "non_fungible_include_nfids": true,
//             "explicit_metadata": [
//                 "name",
//                 "description"
//             ]
//         }
//     });

//     debug_println!("{:?}", addresses);

//     let response = client.post(url).json(&body).send().await?;

//     debug_println!("{:?}", &response);

//     let decoded = decode(response).await?;

//     #[cfg(test)]
//     {
//         use std::fs::File;
//         use std::io::Write;
//         let mut file = File::create("./entity_details.json").unwrap();
//         let value: serde_json::Value = serde_json::from_str(&decoded).unwrap();
//         let formatted = serde_json::to_string_pretty(&value).unwrap();
//         file.write_all(&formatted.as_bytes()).unwrap();

//         Ok(formatted)
//     }

//     #[cfg(not(test))]
//     Ok(decoded)
// }

// pub async fn get_details_for_accounts(
//     client: &RadixDltRequestClient,
//     addresses: &[&str],
//     network: Network,
// ) -> Result<AccountsDetails, RadixDltRequestError> {
//     let entity_details = get_entity_details(client, addresses, network).await?;

//     let component_details: AccountsDetails = serde_json::from_str(&entity_details)?;

//     Ok(component_details)
// }

// pub async fn get_transactions_for_account(
//     client: &RadixDltRequestClient,
//     account_addresses: &AccountAddress,
//     network: Network,
//     from_state_version: u64,
// ) -> Result<TransactionsResponse, RadixDltRequestError> {
//     let decoded_response = get_transactions_for_affected_entity(
//         client,
//         &[account_addresses.as_str()],
//         network,
//         from_state_version,
//     )
//     .await?;

//     let transactions: TransactionsResponse = serde_json::from_str(&decoded_response)?;

//     Ok(transactions)
// }

pub async fn get_accounts_details(
    client: radix_gateway_sdk::Client,
    addresses: &[&str],
) -> Result<StateEntityDetailsResponse, radix_gateway_sdk::Error> {
    let opt_ins = StateEntityDetailsOptIns {
        non_fungible_include_nfids: Some(true),
        explicit_metadata: Some(vec!["symbol".to_string()]),
        ..Default::default()
    };

    client
        .get_inner_client()
        .state_entity_details(addresses, ResourceAggregationLevel::Vault, opt_ins)
        .into_future()
        .await
}

async fn decode(response: reqwest::Response) -> Result<String, RadixDltRequestError> {
    let bytes = response.bytes().await?;

    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut string = String::with_capacity(bytes.len());
    decoder.read_to_string(&mut string)?;
    Ok(string)
}

// #[cfg(test)]
// mod test {
//     use std::str::FromStr;

//     use super::*;

//     #[tokio::test]
//     async fn test_api_get_all_fungibles_for_component() {
//         let radixrequestor =
//             RadixDltRequestClient::new().expect("Unable to construct request builder");
//         let component = AccountAddress::from_str(
//             "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
//         )
//         .expect("Could not create account address");

//         let response =
//             get_all_fungibles_for_account(&radixrequestor, &component, Network::Mainnet).await;

//         match response {
//             Ok(_) => {}
//             Err(err) => panic!("{err}"),
//         }
//     }

//     #[tokio::test]
//     async fn test_api_get_all_nfts_for_component() {
//         let radixrequestor =
//             RadixDltRequestClient::new().expect("Unable to construct request builder");
//         let component = AccountAddress::from_str(
//             "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
//         )
//         .expect("Unable to create account address");

//         let response =
//             get_all_nfts_for_account(&radixrequestor, &component, Network::Mainnet).await;

//         match response {
//             Ok(_) => {}
//             Err(err) => panic!("{err}"),
//         }
//     }

//     #[tokio::test]
//     async fn test_api_get_fungible_vaults() {
//         let radixrequestor =
//             RadixDltRequestClient::new().expect("Unable to construct request builder");
//         let component = AccountAddress::from_str(
//             "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
//         )
//         .expect("Unable to create account address");
//         let resource = ResourceAddress::from_str(
//             "resource_rdx1t5ywq4c6nd2lxkemkv4uzt8v7x7smjcguzq5sgafwtasa6luq7fclq",
//         )
//         .expect("Unable to create resource address");

//         let response = get_fungible_vaults_for_account(
//             &radixrequestor,
//             &component,
//             resource,
//             Network::Mainnet,
//         )
//         .await;

//         match response {
//             Ok(_) => {}
//             Err(err) => panic!("{err}"),
//         }
//     }

//     #[tokio::test]
//     async fn test_api_get_accounts_details() {
//         let radixrequestor =
//             RadixDltRequestClient::new().expect("Unable to construct request builder");
//         let accounts = [
//             "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax".to_owned(),
//             "account_rdx16y60m8p2lxl72rdqcxh6wj270ckku7e3hrr6fra05f9p34zlqwgd0k".to_owned(),
//             "account_rdx12x3el2u8alzssnppe6k76e7l3x5rzuk3p8g8cdu3ayuyse7mn4pt75".to_owned(),
//             "account_rdx168qaecnnf2uyp5w995npdxv4em9wezy8zmpc0zxnvjyd3uf25u959e".to_owned(),
//             "account_rdx168nr5dwmll4k2x5apegw5dhrpejf3xac7khjhgjqyg4qddj9tg9v4d".to_owned(),
//         ];

//         let account_addresses = accounts
//             .iter()
//             .map(|address| address.as_str())
//             .collect::<Vec<&str>>();

//         let response = get_details_for_accounts(
//             &radixrequestor,
//             account_addresses.as_slice(),
//             Network::Mainnet,
//         )
//         .await;

//         match response {
//             Ok(accounts_response) => {
//                 assert_eq!(accounts_response.items.len(), accounts.len())
//             }
//             Err(err) => panic!("{err}"),
//         }
//     }

//     #[tokio::test]
//     async fn test_api_get_transactions_for_accounts() {
//         let radixrequestor =
//             RadixDltRequestClient::new().expect("Unable to construct request builder");
//         let account = AccountAddress::from_str(
//             "account_rdx16y60m8p2lxl72rdqcxh6wj270ckku7e3hrr6fra05f9p34zlqwgd0k",
//         )
//         .unwrap();

//         let response =
//             get_transactions_for_account(&radixrequestor, &account, Network::Mainnet, 1).await;

//         match response {
//             Ok(accounts_response) => {
//                 // assert_eq!(accounts_response.items.len(), accounts.len())
//             }
//             Err(err) => panic!("{err}"),
//         }
//     }
// }
