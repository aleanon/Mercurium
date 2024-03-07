use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Network {
    #[default]
    Mainnet,
    Stokenet,
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
