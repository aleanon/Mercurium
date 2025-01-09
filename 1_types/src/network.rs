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
