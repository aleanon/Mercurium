use deps::*;

use std::collections::HashMap;
use std::str::FromStr;

use debug_print::debug_println;
use radix_gateway_sdk::generated::model::{
    StateEntityDetailsResponseItem, StateEntityFungiblesPageResponse,
    StateEntityNonFungiblesPageResponse, StateNonFungibleDataResponse,
};

use types::address::{AccountAddress, ResourceAddress};
use types::assets::{FungibleAsset, NFT, NFTs, NonFungibleAsset};
use types::response_models::non_fungible_id_data::NFIdData;
use types::response_models::{
    FungibleCollectionItemGlobal, MetaDataStringArrayValue, MetaDataStringValue,
    NonFungibleCollectionItemVaultAggregated, ResourceDetails,
};
use types::{Resource, debug_info};

/// Returns two tuples, first with the resource_address and resource and the second with the resource_address and url to the resources icon
/// Returns None if the resource_address conversion failes.
pub fn parse_resource_details_response(
    response: StateEntityDetailsResponseItem,
) -> Option<(ResourceAddress, (Resource, String))> {
    let resource_address = ResourceAddress::from_str(&response.address).ok()?;

    let (current_supply, divisibility) = response
        .details
        .and_then(|details| {
            serde_json::from_value::<ResourceDetails>(details.0).map(|details| (details.total_supply, details.divisibility))
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
                FungibleAsset::new(account_address, fungible.amount, resource_address.clone());

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
            if !collection_item.vaults.items.is_empty() {
                let vault = collection_item.vaults.items.remove(0);
                if vault.last_updated_at_state_version < last_updated_at_state_version {
                    return None;
                }

                let vault_address = vault.vault_address;

                let asset =
                    NonFungibleAsset::new(account_address, NFTs::new(), resource_address.clone());

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
                .data.map(|data| serde_json::from_value::<NFIdData>(data.programmatic_json.0)
                            .unwrap_or_else(|err| {
                                debug_println!(
                                    "{}{}:{}",
                                    debug_info!("Failed to parse non fungible data for id "),
                                    id,
                                    err
                                );
                                NFIdData { fields: Vec::new() }
                            }))
                .unwrap();

            NFT::from_nfid_data(id, nft_data.fields)
        })
        .collect::<Vec<NFT>>()
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

use chrono::{DateTime, Datelike, Timelike, Utc};
use radix_gateway_sdk::generated::model::CommittedTransactionInfo;
use types::address::{Address, TransactionAddress};
use types::{BalanceChange, TimeStamp, Transaction, TransactionId};

/// Maps a committed gateway transaction into a domain [`Transaction`] for the given account,
/// extracting that account's fungible balance changes. Returns `None` if the transaction lacks a
/// confirmed time or a (well-formed) intent hash.
pub fn parse_transaction(
    info: &CommittedTransactionInfo,
    account: &AccountAddress,
) -> Option<Transaction> {
    let intent_hash = info.intent_hash.as_ref()?;
    let tx_address = TransactionAddress::from_str(intent_hash).ok()?;
    let confirmed_at = info.confirmed_at.as_ref()?;
    let timestamp = timestamp_from_datetime(confirmed_at);
    let tx_id = TransactionId::new(account, &tx_address);

    let balance_changes = info
        .balance_changes
        .as_ref()
        .map(|changes| {
            changes
                .fungible_balance_changes
                .iter()
                .filter(|change| change.entity_address == account.as_str())
                .filter_map(|change| {
                    let resource = ResourceAddress::from_str(&change.resource_address).ok()?;
                    Some(BalanceChange::new(
                        tx_id.clone(),
                        account.clone(),
                        resource,
                        None,
                        Some(change.balance_change.clone()),
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    let message = info
        .message
        .as_ref()
        .and_then(|value| value.as_str().map(String::from));

    Some(Transaction::new(
        timestamp,
        info.state_version,
        balance_changes,
        account,
        tx_address,
        message,
    ))
}

fn timestamp_from_datetime(datetime: &DateTime<Utc>) -> TimeStamp {
    TimeStamp::new(
        datetime.year() as u16,
        datetime.month() as u8,
        datetime.day() as u8,
        datetime.hour() as u8,
        datetime.minute() as u8,
        datetime.second() as u8,
    )
}

#[cfg(test)]
mod transaction_parse_tests {
    use super::*;

    const ACCOUNT: &str = "account_rdx128ykx9agh0maq8nw6h6pzmltmaexts0xf24sledqp44x5cdec0uqjj";
    const XRD: &str = "resource_rdx1t4h4396mukhpzdrr5sfvegjsxl8q7a34q2vkt4quxcxahna8fucuz4";

    fn valid_txid() -> String {
        // txid_ + rdx1 + 58 chars matching the [a-z0-9] address regex.
        format!("txid_rdx1{}", "a".repeat(58))
    }

    fn info_json(intent_hash: Option<String>, confirmed: bool) -> String {
        let intent = intent_hash
            .map(|h| format!("\"intent_hash\": \"{h}\","))
            .unwrap_or_default();
        let confirmed_at = if confirmed {
            "\"confirmed_at\": \"2024-03-07T14:40:35Z\","
        } else {
            ""
        };
        format!(
            r#"{{
                "epoch": 1000,
                "round": 1,
                "round_timestamp": "2024-03-07T14:40:35Z",
                "state_version": 5000,
                "transaction_status": "CommittedSuccess",
                {confirmed_at}
                {intent}
                "message": "hello",
                "balance_changes": {{
                    "fungible_balance_changes": [
                        {{ "balance_change": "-10", "entity_address": "{ACCOUNT}", "resource_address": "{XRD}" }}
                    ],
                    "fungible_fee_balance_changes": [],
                    "non_fungible_balance_changes": []
                }}
            }}"#
        )
    }

    fn parse(json: &str) -> Option<Transaction> {
        let info: CommittedTransactionInfo = serde_json::from_str(json).unwrap();
        let account = AccountAddress::from_str(ACCOUNT).unwrap();
        parse_transaction(&info, &account)
    }

    #[test]
    fn parses_a_committed_transaction() {
        let tx = parse(&info_json(Some(valid_txid()), true)).expect("parses");
        assert_eq!(tx.state_version, 5000);
        assert_eq!(tx.message.as_deref(), Some("hello"));
        assert_eq!(tx.timestamp.year(), 2024);
        assert_eq!(tx.balance_changes.len(), 1);
        assert_eq!(tx.balance_changes[0].amount.as_deref(), Some("-10"));
    }

    #[test]
    fn returns_none_without_intent_hash_or_confirmation() {
        assert!(parse(&info_json(None, true)).is_none());
        assert!(parse(&info_json(Some(valid_txid()), false)).is_none());
    }
}
