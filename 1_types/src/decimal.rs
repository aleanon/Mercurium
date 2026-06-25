use deps::*;

use async_sqlite::rusqlite;
use std::{
    fmt::{self, Display},
    ops::{Deref, DerefMut},
    str::FromStr,
};

use scrypto::math::Decimal as RadixDecimal;

///Newtype wrapper around the radix engine Decimal type to implement To/From sql
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_and_into_radix_decimal() {
        let radix = RadixDecimal::from(5);
        let decimal: Decimal = radix.into();
        assert_eq!(*decimal, RadixDecimal::from(5));
        let back: RadixDecimal = decimal.into();
        assert_eq!(back, RadixDecimal::from(5));
    }

    #[test]
    fn display_matches_inner_value() {
        let decimal = Decimal(RadixDecimal::from_str("123.45").unwrap());
        assert_eq!(format!("{decimal}"), "123.45");
    }

    #[test]
    fn deref_exposes_inner_decimal() {
        let decimal = Decimal(RadixDecimal::from(10));
        assert_eq!(*decimal, RadixDecimal::from(10));
    }
}
