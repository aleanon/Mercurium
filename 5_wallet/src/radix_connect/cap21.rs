//! CAP-21 wallet-interaction wire model (Radix Connect).
//!
//! Serde model for the JSON a dApp sends/receives over the relay, plus conversion to the
//! internal [`super::WalletInteraction`] domain type.
//!
//! The field names, discriminators and nesting are **reconciled against the official
//! `@radixdlt/radix-dapp-toolkit` schema** (`packages/dapp-toolkit/src/schemas/index.ts`):
//! `interactionId`; `metadata { version: 2, networkId, dAppDefinitionAddress, origin }`; the items
//! union `{ unauthorizedRequest | authorizedRequest | transaction }`; the auth discriminators
//! `loginWithChallenge` / `loginWithoutChallenge` / `usePersona`; and
//! `transaction.send { transactionManifest, version, blobs?, message? }`.

use deps::*;

use serde::{Deserialize, Serialize};
use types::Network;

use super::{ConnectError, DappMetadata, WalletInteraction, WalletRequest, WalletResponse};

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Serializes a [`WalletResponse`] into the CAP-21 response JSON sent back to the dApp.
///
/// The success/failure discriminator and `interactionId` match the official schema; the inner
/// item/proof field names are the spec-sensitive part to confirm against a live dApp (the proof
/// is an Ed25519 ROLA signature over the dApp-bound challenge).
pub fn response_to_cap21_json(
    interaction_id: &str,
    response: &WalletResponse,
) -> Result<String, ConnectError> {
    let value = match response {
        WalletResponse::Auth {
            persona_identity_address,
            proof,
        } => serde_json::json!({
            "discriminator": "success",
            "interactionId": interaction_id,
            "items": {
                "discriminator": "authorizedRequest",
                "auth": {
                    "discriminator": "loginWithChallenge",
                    "persona": { "identityAddress": persona_identity_address },
                    "proof": {
                        "publicKey": to_hex(&proof.public_key.0),
                        "signature": to_hex(&proof.signature.0),
                        "curve": "curve25519"
                    }
                }
            }
        }),
        WalletResponse::Transaction { intent_hash_id } => serde_json::json!({
            "discriminator": "success",
            "interactionId": interaction_id,
            "items": {
                "discriminator": "transaction",
                "send": { "transactionIntentHash": intent_hash_id }
            }
        }),
        WalletResponse::Rejected { reason } => serde_json::json!({
            "discriminator": "failure",
            "interactionId": interaction_id,
            "error": "rejectedByUser",
            "message": reason
        }),
    };

    serde_json::to_string(&value)
        .map_err(|e| ConnectError::Transport(format!("failed to serialize CAP-21 response: {e}")))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WalletInteractionWire {
    pub interaction_id: String,
    pub metadata: MetadataWire,
    pub items: ItemsWire,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataWire {
    pub version: u32,
    /// Radix network id (1 = mainnet, 2 = stokenet).
    pub network_id: u8,
    pub d_app_definition_address: String,
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "discriminator", rename_all = "camelCase")]
pub enum ItemsWire {
    #[serde(rename = "authorizedRequest")]
    AuthorizedRequest { auth: AuthWire },
    #[serde(rename = "transaction")]
    Transaction { send: SendTransactionWire },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "discriminator", rename_all = "camelCase")]
pub enum AuthWire {
    #[serde(rename = "loginWithChallenge")]
    LoginWithChallenge { challenge: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SendTransactionWire {
    pub transaction_manifest: String,
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blobs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

fn network_from_id(id: u8) -> Result<Network, ConnectError> {
    Network::from_id(id as u32)
        .map_err(|id| ConnectError::Transport(format!("unsupported network id {id}")))
}

fn decode_challenge(hex: &str) -> Result<[u8; 32], ConnectError> {
    if hex.len() != 64 {
        return Err(ConnectError::Transport(
            "challenge must be 32 bytes (64 hex chars)".to_string(),
        ));
    }
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let s = std::str::from_utf8(chunk)
            .map_err(|_| ConnectError::Transport("invalid challenge hex".to_string()))?;
        out[i] = u8::from_str_radix(s, 16)
            .map_err(|_| ConnectError::Transport("invalid challenge hex".to_string()))?;
    }
    Ok(out)
}

impl WalletInteractionWire {
    /// Parses a CAP-21 JSON string into the wire model.
    pub fn from_json(json: &str) -> Result<Self, ConnectError> {
        serde_json::from_str(json)
            .map_err(|e| ConnectError::Transport(format!("invalid CAP-21 json: {e}")))
    }

    /// Serializes the wire model back to CAP-21 JSON.
    pub fn to_json(&self) -> Result<String, ConnectError> {
        serde_json::to_string(self)
            .map_err(|e| ConnectError::Transport(format!("failed to serialize CAP-21: {e}")))
    }

    /// Converts the wire model into the internal interaction domain type used by the dispatcher.
    pub fn into_interaction(self) -> Result<WalletInteraction, ConnectError> {
        let network = network_from_id(self.metadata.network_id)?;
        let metadata = DappMetadata {
            dapp_definition_address: self.metadata.d_app_definition_address,
            origin: self.metadata.origin,
            network,
        };

        let request = match self.items {
            ItemsWire::AuthorizedRequest {
                auth: AuthWire::LoginWithChallenge { challenge },
            } => WalletRequest::AuthLogin {
                challenge: decode_challenge(&challenge)?,
            },
            ItemsWire::Transaction { send } => WalletRequest::SendTransaction {
                manifest_rtm: send.transaction_manifest,
                message: send.message,
            },
        };

        Ok(WalletInteraction {
            interaction_id: self.interaction_id,
            metadata,
            request,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_transaction_interaction() {
        let json = r#"{
            "interactionId": "abc-123",
            "metadata": {
                "version": 2,
                "networkId": 2,
                "dAppDefinitionAddress": "account_tdx_2_12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66",
                "origin": "https://dapp.example"
            },
            "items": {
                "discriminator": "transaction",
                "send": {
                    "transactionManifest": "CALL_METHOD Address(\"account_tdx_2_1\") \"withdraw\";",
                    "version": 1
                }
            }
        }"#;

        let wire = WalletInteractionWire::from_json(json).expect("parses");
        let interaction = wire.into_interaction().expect("converts");

        assert_eq!(interaction.interaction_id, "abc-123");
        assert_eq!(interaction.metadata.network, Network::Stokenet);
        assert!(matches!(
            interaction.request,
            WalletRequest::SendTransaction { .. }
        ));
    }

    #[test]
    fn parses_a_login_interaction() {
        let challenge_hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        let json = format!(
            r#"{{
                "interactionId": "login-1",
                "metadata": {{
                    "version": 2,
                    "networkId": 1,
                    "dAppDefinitionAddress": "account_rdx12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66",
                    "origin": "https://dapp.example"
                }},
                "items": {{
                    "discriminator": "authorizedRequest",
                    "auth": {{ "discriminator": "loginWithChallenge", "challenge": "{challenge_hex}" }}
                }}
            }}"#
        );

        let interaction = WalletInteractionWire::from_json(&json)
            .unwrap()
            .into_interaction()
            .unwrap();

        match interaction.request {
            WalletRequest::AuthLogin { challenge } => {
                assert_eq!(challenge[0], 0x00);
                assert_eq!(challenge[31], 0xff);
            }
            _ => panic!("expected an auth login request"),
        }
    }

    #[test]
    fn wire_model_json_roundtrips() {
        let wire = WalletInteractionWire {
            interaction_id: "rt-1".to_string(),
            metadata: MetadataWire {
                version: 2,
                network_id: 2,
                d_app_definition_address: "account_tdx_2_1xyz".to_string(),
                origin: "https://x.example".to_string(),
            },
            items: ItemsWire::Transaction {
                send: SendTransactionWire {
                    transaction_manifest: "CALL_METHOD;".to_string(),
                    version: 1,
                    blobs: None,
                    message: Some("hi".to_string()),
                },
            },
        };

        let json = wire.to_json().unwrap();
        let parsed = WalletInteractionWire::from_json(&json).unwrap();
        assert_eq!(wire, parsed);
    }
}

#[cfg(test)]
mod negative_tests {
    use super::*;

    #[test]
    fn login_with_malformed_challenge_is_rejected() {
        let json = r#"{
            "interactionId": "x",
            "metadata": { "version": 2, "networkId": 1,
                "dAppDefinitionAddress": "account_rdx1", "origin": "https://x" },
            "items": { "discriminator": "authorizedRequest",
                "auth": { "discriminator": "loginWithChallenge", "challenge": "00" } }
        }"#;
        let result = WalletInteractionWire::from_json(json).unwrap().into_interaction();
        assert!(result.is_err(), "a 1-byte challenge must be rejected");
    }

    #[test]
    fn unsupported_network_id_is_rejected() {
        let json = r#"{
            "interactionId": "x",
            "metadata": { "version": 2, "networkId": 99,
                "dAppDefinitionAddress": "account_rdx1", "origin": "https://x" },
            "items": { "discriminator": "transaction",
                "send": { "transactionManifest": "CALL_METHOD;", "version": 1 } }
        }"#;
        let result = WalletInteractionWire::from_json(json).unwrap().into_interaction();
        assert!(result.is_err(), "network id 99 is not supported");
    }

    #[test]
    fn invalid_json_is_rejected() {
        assert!(WalletInteractionWire::from_json("{ not json").is_err());
    }
}
