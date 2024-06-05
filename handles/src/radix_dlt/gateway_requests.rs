use std::{future::IntoFuture, io::Read, ops::Add, sync::Arc};

use flate2::bufread::GzDecoder;
use iced::futures::future::join_all;
use radix_gateway_sdk::generated::model::{
    ResourceAggregationLevel, StateEntityDetailsOptIns, StateEntityDetailsResponse,
    StateNonFungibleDataResponse,
};
use types::{address::AddressTrait, radix_request_client::RadixDltRequestError};

pub async fn get_entities_details2(
    client: &radix_gateway_sdk::Client,
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

pub async fn get_entities_details(
    client: Arc<radix_gateway_sdk::Client>,
    addresses: &[String],
) -> Result<StateEntityDetailsResponse, radix_gateway_sdk::Error> {
    let tasks = addresses.chunks(100).map(|chunk| {
        let client = client.clone();
        let chunk = chunk.to_owned();
        let opt_ins = StateEntityDetailsOptIns {
            non_fungible_include_nfids: Some(true),
            explicit_metadata: Some(vec!["symbol".to_string()]),
            ..Default::default()
        };

        tokio::task::spawn(async move {
            let addresses = chunk
                .iter()
                .map(|address| address.as_str())
                .collect::<Vec<&str>>();

            client
                .get_inner_client()
                .state_entity_details(
                    addresses.as_slice(),
                    ResourceAggregationLevel::Vault,
                    opt_ins,
                )
                .into_future()
                .await
        })
    });

    join_all(tasks)
        .await
        .into_iter()
        .filter_map(|join_result| join_result.ok())
        .reduce(|acc, response_result| match response_result {
            Ok(response) => match acc {
                Ok(mut ok_acc) => {
                    ok_acc.items.extend(response.items.into_iter());
                    Ok(ok_acc)
                }
                Err(_) => Ok(response),
            },
            Err(e) => acc.map_err(|_| e),
        })
        .unwrap_or(Err(radix_gateway_sdk::Error::NetworkInvalid))
}

pub async fn get_non_fungible_data(
    client: Arc<radix_gateway_sdk::Client>,
    resource_address: &str,
    non_fungible_ids: &[String],
) -> Result<StateNonFungibleDataResponse, radix_gateway_sdk::Error> {
    let tasks = non_fungible_ids.chunks(100).map(|chunk| {
        let client = client.clone();
        let resource_address = resource_address.to_string();
        let chunk = chunk.to_owned();

        tokio::task::spawn(async move {
            let addresses: Vec<&str> = chunk.iter().map(|address| address.as_str()).collect();
            client
                .get_inner_client()
                .non_fungible_data(addresses.as_slice(), resource_address.as_str())
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

async fn decode(response: reqwest::Response) -> Result<String, RadixDltRequestError> {
    let bytes = response.bytes().await?;

    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut string = String::with_capacity(bytes.len());
    decoder.read_to_string(&mut string)?;
    Ok(string)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use types::{AccountAddress, ResourceAddress};

    use super::*;

    #[tokio::test]
    async fn test_get_entities_details() {
        let client = Arc::new(
            radix_gateway_sdk::Client::new(radix_gateway_sdk::Network::Mainnet, None, None)
                .unwrap(),
        );

        let account_address = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .unwrap();

        let account_details = get_entities_details(client.clone(), &[account_address.to_string()])
            .await
            .unwrap();

        assert_eq!(account_details.items.len(), 1);
        assert!(account_details.items[0].fungible_resources.is_some());
        assert!(account_details.items[0].non_fungible_resources.is_some());

        let resource_address = ResourceAddress::from_str(
            "resource_rdx1t4h4396mukhpzdrr5sfvegjsxl8q7a34q2vkt4quxcxahna8fucuz4",
        )
        .unwrap();
        let resource_details =
            get_entities_details(client.clone(), &[resource_address.to_string()])
                .await
                .unwrap();

        assert_eq!(resource_details.items.len(), 1);
    }
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
