use std::io::Read;

use debug_print::debug_println;
use flate2::bufread::GzDecoder;
use serde_json::json;
use types::{
    radix_request_client::{RadixDltRequestClient, RadixDltRequestError},
    response_models::{
        AllFungiblesResponse, AllNFTsResponse, EntityDetailsResponse, FungibleVaultsResponse,
    },
    AccountAddress, Network, ResourceAddress,
};

use crate::radix_dlt::urls::{GET_ENTITY_DETAILS_MAINNET, GET_ENTITY_DETAILS_STOKENET};

use super::urls::{
    GET_COMPONENT_FUNGIBLES_MAINNET, GET_COMPONENT_FUNGIBLES_STOKENET, GET_COMPONENT_NFTS_MAINNET,
    GET_COMPONENT_NFTS_STOKENET, GET_FUNGIBLE_VAULTS_MAINNET, GET_FUNGIBLE_VAULTS_STOKENET,
};

pub async fn get_all_fungibles_for_account(
    client: RadixDltRequestClient,
    component: AccountAddress,
    network: Network,
) -> Result<AllFungiblesResponse, RadixDltRequestError> {
    let url = match network {
        Network::Mainnet => GET_COMPONENT_FUNGIBLES_MAINNET,
        Network::Stokenet => GET_COMPONENT_FUNGIBLES_STOKENET,
    };

    let body = json!({
        "address": component.as_str(),
    });

    let response = client.post(url).json(&body).send().await?;

    let decoded = decode(response).await?;

    let all_fungibles: AllFungiblesResponse = serde_json::from_str(&decoded)?;

    Ok(all_fungibles)
}

pub async fn get_all_nfts_for_account(
    client: RadixDltRequestClient,
    component: AccountAddress,
    network: Network,
) -> Result<AllNFTsResponse, RadixDltRequestError> {
    let url = match network {
        Network::Mainnet => GET_COMPONENT_NFTS_MAINNET,
        Network::Stokenet => GET_COMPONENT_NFTS_STOKENET,
    };

    let body = json!({
        "address": component.as_str()
    });

    let response = client.post(url).json(&body).send().await?;

    let decoded = decode(response).await?;
    let all_non_fungibles: AllNFTsResponse = serde_json::from_str(&decoded)?;

    Ok(all_non_fungibles)
}

pub async fn get_fungible_vaults_for_account(
    client: RadixDltRequestClient,
    component: AccountAddress,
    resource: ResourceAddress,
    network: Network,
) -> Result<FungibleVaultsResponse, RadixDltRequestError> {
    let url = match network {
        Network::Mainnet => GET_FUNGIBLE_VAULTS_MAINNET,
        Network::Stokenet => GET_FUNGIBLE_VAULTS_STOKENET,
    };

    let body = json!({
        "address": component.as_str(),
        "resource_address": resource.as_str(),
    });

    let response = client.post(url).json(&body).send().await?;

    let decoded = decode(response).await?;
    let fungible_vaults: FungibleVaultsResponse = serde_json::from_str(&decoded)?;

    Ok(fungible_vaults)
}

pub async fn get_entity_details(
    client: RadixDltRequestClient,
    addresses: &[&str],
    network: Network,
) -> Result<EntityDetailsResponse, RadixDltRequestError> {
    let url = match network {
        Network::Mainnet => GET_ENTITY_DETAILS_MAINNET,
        Network::Stokenet => GET_ENTITY_DETAILS_STOKENET,
    };

    let body = json!({
        "addresses": addresses,
        "aggregation_level": "Vault",
        "opt_ins": {
            "ancestor_identities": true,
            "component_royalty_vault_balance": true,
            "package_royalty_vault_balance": true,
            "non_fungible_include_nfids": true,
            "explicit_metadata": [
                "name",
                "description"
            ]
        }
    });

    debug_println!("{:?}", addresses);

    let response = client.post(url).json(&body).send().await?;

    debug_println!("{:?}", &response);

    let decoded = decode(response).await?;

    #[cfg(test)]
    {
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create("./json.json").unwrap();
        let value: serde_json::Value = serde_json::from_str(&decoded).unwrap();
        let formatted = serde_json::to_string_pretty(&value).unwrap();
        file.write_all(&formatted.as_bytes()).unwrap();
    }

    let component_details: EntityDetailsResponse = serde_json::from_str(&decoded)?;

    Ok(component_details)
}

async fn decode(response: reqwest::Response) -> Result<String, RadixDltRequestError> {
    let bytes = response.bytes().await?;

    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut string = String::with_capacity(bytes.len());
    decoder.read_to_string(&mut string)?;
    Ok(string)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn test_api_get_all_fungibles_for_component() {
        let radixrequestor =
            RadixDltRequestClient::new().expect("Unable to construct request builder");
        let component = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .expect("Could not create account address");

        let response =
            get_all_fungibles_for_account(radixrequestor, component, Network::Mainnet).await;

        match response {
            Ok(_) => {}
            Err(err) => panic!("{err}"),
        }
    }

    #[tokio::test]
    async fn test_api_get_all_nfts_for_component() {
        let radixrequestor =
            RadixDltRequestClient::new().expect("Unable to construct request builder");
        let component = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .expect("Unable to create account address");

        let response = get_all_nfts_for_account(radixrequestor, component, Network::Mainnet).await;

        match response {
            Ok(_) => {}
            Err(err) => panic!("{err}"),
        }
    }

    #[tokio::test]
    async fn test_api_get_fungible_vaults() {
        let radixrequestor =
            RadixDltRequestClient::new().expect("Unable to construct request builder");
        let component = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .expect("Unable to create account address");
        let resource = ResourceAddress::from_str(
            "resource_rdx1t5ywq4c6nd2lxkemkv4uzt8v7x7smjcguzq5sgafwtasa6luq7fclq",
        )
        .expect("Unable to create resource address");

        let response =
            get_fungible_vaults_for_account(radixrequestor, component, resource, Network::Mainnet)
                .await;

        match response {
            Ok(_) => {}
            Err(err) => panic!("{err}"),
        }
    }

    #[tokio::test]
    async fn test_api_get_entity_details() {
        let radixrequestor =
            RadixDltRequestClient::new().expect("Unable to construct request builder");
        let component =
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax".to_owned();

        let response = get_entity_details(radixrequestor, &[&component], Network::Mainnet).await;

        match response {
            Ok(_) => {}
            Err(err) => panic!("{err}"),
        }
    }
}
