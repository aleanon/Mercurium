use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Network {
    #[default]
    Mainnet,
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
