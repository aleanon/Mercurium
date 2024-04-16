
pub mod urls;

use std::io::Read;
use debug_print::debug_println;
use serde_json::json;
use urls::*;
use flate2::bufread::GzDecoder;
use reqwest::header::{CONTENT_TYPE, USER_AGENT, ACCEPT, ACCEPT_ENCODING, CONNECTION, HeaderMap};

use thiserror::Error;


use types::{response_models::{AllFungiblesResponse, AllNFTsResponse, EntityDetailsResponse, FungibleVaultsResponse}, AccountAddress, Network, ResourceAddress};



#[derive(Debug, Error)]
pub enum RadixDltRequestError {
    #[error("Unable to create request header, source: {0}")]
    FailedToParseHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Unable to build reqwest Client, source: {0}")]
    FailedToBuildClient(reqwest::Error),
    #[error("Request failed, source: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Unable to decode response, source: {0}")]
    DecodeError(#[from] std::io::Error),
    #[error("Unable to parse response from json, source: {0}")]
    ParseError(#[from] serde_json::Error),
}


#[derive(Debug)]
pub struct RadixDltRequestBuilder {
    client: reqwest::Client,
    network: Network,   // Should be moved to application settings
}


impl RadixDltRequestBuilder {
    pub fn new() -> Result<Self, RadixDltRequestError> {
        let mut headers = HeaderMap::with_capacity(5);
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(USER_AGENT, "PostmanRuntime/7.34.0".parse()?);
        headers.insert(ACCEPT, "*/*".parse()?);
        headers.insert(ACCEPT_ENCODING, "gzip".parse()?);
        headers.insert(CONNECTION, "keep-alive".parse()?);

        let client = reqwest::Client::builder().default_headers(headers).build()
            .map_err(|err| RadixDltRequestError::FailedToBuildClient(err))?;

        Ok(Self { 
            client,
            network: Network::Mainnet
        })
    }

    pub fn client(&self) -> reqwest::Client {
        self.client.clone()
    }

    pub async fn get_all_fungibles_for_component(&self, component: AccountAddress) -> Result<AllFungiblesResponse, RadixDltRequestError> {
        let url = match self.network {
            Network::Mainnet => GET_COMPONENT_FUNGIBLES_MAINNET,
            Network::Stokenet => GET_COMPONENT_FUNGIBLES_STOKENET,
        };

        let body = json!({
            "address": component.as_str(),
        });

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;
                
            let decoded = Self::decode(response).await?; 

            let all_fungibles: AllFungiblesResponse = serde_json::from_str(&decoded)?;

        Ok(all_fungibles)
    }

    pub async fn get_all_nfts_for_component(&self, component: AccountAddress) -> Result<AllNFTsResponse, RadixDltRequestError> {
        let url = match self.network {
            Network::Mainnet => GET_COMPONENT_NFTS_MAINNET,
            Network::Stokenet => GET_COMPONENT_NFTS_STOKENET,
        };

        let body = json!({
            "address": component.as_str()
        });

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;
                
        let decoded = Self::decode(response).await?;
        let all_non_fungibles: AllNFTsResponse = serde_json::from_str(&decoded)?;

        Ok(all_non_fungibles)
    }

    pub async fn get_fungible_vaults_for_component(&self, component: AccountAddress, resource: ResourceAddress) -> Result<FungibleVaultsResponse, RadixDltRequestError> {
        let url = match self.network {
            Network::Mainnet => GET_FUNGIBLE_VAULTS_MAINNET,
            Network::Stokenet => GET_FUNGIBLE_VAULTS_STOKENET,
        };

        let body = json!({
            "address": component.as_str(),
            "resource_address": resource.as_str(),
        });

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;
                
        let decoded = Self::decode(response).await?;
        let fungible_vaults: FungibleVaultsResponse = serde_json::from_str(&decoded)?;

        Ok(fungible_vaults)
    }

    pub async fn get_entity_details(&self, addresses: &[&str]) -> Result<EntityDetailsResponse, RadixDltRequestError> {
        let url = match self.network {
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
        #[cfg(debug_assertions)]
        println!("{:?}", addresses);

        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;

        debug_println!("{:?}", &response);

        let decoded = Self::decode(response).await?;

        #[cfg(test)]
        {
            use std::io::Write;
            use std::fs::File;
            let mut file = File::create("./json.json").unwrap();
            let value:serde_json::Value = serde_json::from_str(&decoded).unwrap();
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
}



#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn test_api_get_all_fungibles_for_component() {
        let radixrequestor = RadixDltRequestBuilder::new().expect("Unable to construct request builder");
        let component = AccountAddress::from_str("account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax")
            .expect("Could not create account address");

        let response = radixrequestor.get_all_fungibles_for_component(component).await;

        match response {
            Ok(_) => {},
            Err(err) => panic!("{err}")
        }
    }

    #[tokio::test]
    async fn test_api_get_all_nfts_for_component() {
        let radixrequestor = RadixDltRequestBuilder::new().expect("Unable to construct request builder");
        let component = AccountAddress::from_str("account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax").expect("Unable to create account address");

        let response = radixrequestor.get_all_nfts_for_component(component).await;

        match response {
            Ok(_) => {},
            Err(err) => panic!("{err}")
        }
    }

    #[tokio::test]
    async fn test_api_get_fungible_vaults() {
        let radixrequestor = RadixDltRequestBuilder::new().expect("Unable to construct request builder");
        let component = AccountAddress::from_str("account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax").expect("Unable to create account address");
        let resource = ResourceAddress::from_str("resource_rdx1t5ywq4c6nd2lxkemkv4uzt8v7x7smjcguzq5sgafwtasa6luq7fclq").expect("Unable to create resource address");

        let response = radixrequestor.get_fungible_vaults_for_component(component, resource).await; 

        match response {
            Ok(_) => {},
            Err(err) => panic!("{err}")
        }
    }

    #[tokio::test]
    async fn test_api_get_entity_details() {
        let radixrequestor = RadixDltRequestBuilder::new().expect("Unable to construct request builder");
        let component = "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax".to_owned();

        let response =  radixrequestor.get_entity_details(&[&component]).await;

        match response {
            Ok(_) => {},
            Err(err) => panic!("{err}")
        }
    }
}