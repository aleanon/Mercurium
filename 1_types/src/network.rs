use async_sqlite::rusqlite;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Network {
    Mainnet,
    #[default]
    Stokenet,
}

impl Network {
    pub const MAINNET_PREFIX: [u8; Self::MAINNET_PREFIX_LENGTH] = [b'r', b'd', b'x', b'1'];
    pub const MAINNET_PREFIX_LENGTH: usize = 4;
    pub const STOKENET_PREFIX: [u8; Self::STOKENET_PREFIX_LENGTH] =
        [b't', b'd', b'x', b'_', b'2', b'_', b'1'];
    pub const STOKENET_PREFIX_LENGTH: usize = 7;
    pub const MAINNET_PREFIX_STR: &'static str = "rdx1";
    pub const STOKENET_PREFIX_STR: &'static str = "tdx_2_1";

    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Mainnet => Self::MAINNET_PREFIX_STR,
            Self::Stokenet => Self::STOKENET_PREFIX_STR,
        }
    }
}

impl Into<scrypto::network::NetworkDefinition> for Network {
    fn into(self) -> scrypto::network::NetworkDefinition {
        match self {
            Self::Mainnet => scrypto::network::NetworkDefinition::mainnet(),
            Self::Stokenet => scrypto::network::NetworkDefinition::stokenet(),
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
            rusqlite::types::ValueRef::Integer(int) => match int {
                1 => Ok(Network::Mainnet),
                2 => Ok(Network::Stokenet),
                int => Err(rusqlite::types::FromSqlError::OutOfRange(int)),
            },
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for Network {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Integer(match self {
                Network::Mainnet => 1,
                Network::Stokenet => 2,
            }),
        ))
    }
}
