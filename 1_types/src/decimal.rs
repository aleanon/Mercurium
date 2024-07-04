use std::{
    fmt::{self, Display},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use scrypto::math::Decimal as RadixDecimal;

///Newtype wrapper around the radix engine Decimal type
#[derive(Debug, Clone)]
pub struct Decimal(pub RadixDecimal);

impl From<RadixDecimal> for Decimal {
    fn from(value: RadixDecimal) -> Self {
        Self(value)
    }
}

impl Into<RadixDecimal> for Decimal {
    fn into(self) -> RadixDecimal {
        self.0
    }
}

impl Deref for Decimal {
    type Target = RadixDecimal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Decimal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl rusqlite::types::FromSql for Decimal {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Text(slice) => Ok(Decimal(
                RadixDecimal::from_str(
                    std::str::from_utf8(slice)
                        .map_err(|_| rusqlite::types::FromSqlError::InvalidType)?,
                )
                .map_err(|_| rusqlite::types::FromSqlError::InvalidType)?,
            )),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for Decimal {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        rusqlite::Result::Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Text(self.0.to_string()),
        ))
    }
}
