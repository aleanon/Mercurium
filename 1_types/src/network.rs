use deps::*;

use async_sqlite::rusqlite;
use scrypto::prelude::NetworkDefinition;
use serde::{Deserialize, Serialize};

use crate::crypto::derivation_path_indexes::{BIP32_NETWORK_ID_MAINNET, BIP32_NETWORK_ID_STOKENET};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Network {
    #[default]
    Mainnet,
    Stokenet,
}

impl Network {
    pub const MAINNET_PREFIX: &'static str = "rdx1";
    pub const STOKENET_PREFIX: &'static str = "tdx_2_1";

    pub fn from_id(id: u32) -> Result<Self, u32> {
        match id {
            BIP32_NETWORK_ID_MAINNET => Ok(Self::Mainnet),
            BIP32_NETWORK_ID_STOKENET => Ok(Self::Stokenet),
            _ => Err(id),
        }
    }

    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Mainnet => Self::MAINNET_PREFIX,
            Self::Stokenet => Self::STOKENET_PREFIX,
        }
    }

    pub fn definition(&self) -> NetworkDefinition {
        match self {
            Self::Mainnet => scrypto::network::NetworkDefinition::mainnet(),
            Self::Stokenet => scrypto::network::NetworkDefinition::stokenet(),
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Self::Mainnet => BIP32_NETWORK_ID_MAINNET,
            Self::Stokenet => BIP32_NETWORK_ID_STOKENET,
        }
    }
}

impl Into<radix_gateway_sdk::Network> for Network {
    fn into(self) -> radix_gateway_sdk::Network {
        match self {
            Network::Mainnet => radix_gateway_sdk::Network::Mainnet,
            Network::Stokenet => radix_gateway_sdk::Network::Stokenet,
        }
    }
}

impl rusqlite::types::FromSql for Network {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Integer(int) => Self::from_id(int as u32)
                .map_err(|id| rusqlite::types::FromSqlError::OutOfRange(id as i64)),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for Network {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Integer(self.id() as i64),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_id_maps_known_networks() {
        assert_eq!(Network::from_id(1), Ok(Network::Mainnet));
        assert_eq!(Network::from_id(2), Ok(Network::Stokenet));
        assert_eq!(Network::from_id(99), Err(99));
    }

    #[test]
    fn id_and_from_id_roundtrip() {
        for network in [Network::Mainnet, Network::Stokenet] {
            assert_eq!(Network::from_id(network.id()), Ok(network));
        }
    }

    #[test]
    fn prefixes_and_ids() {
        assert_eq!(Network::Mainnet.id(), 1);
        assert_eq!(Network::Stokenet.id(), 2);
        assert_eq!(Network::Mainnet.prefix(), "rdx1");
        assert_eq!(Network::Stokenet.prefix(), "tdx_2_1");
    }

    #[test]
    fn default_is_mainnet() {
        assert_eq!(Network::default(), Network::Mainnet);
    }

    #[test]
    fn definition_id_matches() {
        assert_eq!(Network::Mainnet.definition().id, 1);
        assert_eq!(Network::Stokenet.definition().id, 2);
    }
}
