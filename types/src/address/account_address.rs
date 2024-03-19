use super::ParseAddrError;
use scrypto::prelude::indexmap::Equivalent;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::str::FromStr;

const ACC_ADDR_LENGTH: usize = 66;
const ACC_TRUNCATE_LEN: usize = 13;

// #[derive(Clone, Debug, Error)]
// pub enum AddrParseError {
//     #[error("Non ASCII character")]
//     NonAsciiCharacter,
//     #[error("Invalid length, expected: {expected}, found: {found}")]
//     InvalidLength {
//         expected: usize,
//         found: usize,
//     }
// }

// impl std::fmt::Display for AccAddrError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             AccAddrError::InvalidLength { expected, found } => write!(f, "Invalid length, expected: {}, found: {}", expected, found),
//             AccAddrError::NonAsciiCharacter => write!(f, "Non ASCII character"),
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountAddress([u8; ACC_ADDR_LENGTH]);

impl AccountAddress {

    pub fn empty() -> Self {
        Self([b'0'; ACC_ADDR_LENGTH])
    }

    pub fn as_ref(&self) -> &[u8] {
        &self.0
    }

    // Uses unsafe because ``AccountAddress`` can not be created with invalid UTF-8 characters
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }

    pub fn is_empty(&self) -> bool {
        self.0.equivalent(&[b'0'; ACC_ADDR_LENGTH])
    }

    pub fn truncate(&self) -> String {
        let mut truncated:[u8;ACC_TRUNCATE_LEN] = [b'.';ACC_TRUNCATE_LEN];

        truncated[..4].copy_from_slice(&self.0[..4]);
        truncated[ACC_TRUNCATE_LEN-6..].copy_from_slice(&self.0[ACC_ADDR_LENGTH-6..]);

        //Uses unsafe because ``AccountAddress`` can not be created with invalid UTF-8 characters
        unsafe{String::from_utf8_unchecked(truncated.to_vec())}
    }

    pub fn truncate_long(&self) -> String {
        let truncated = [&self.0[..12], &[b'.';3], &self.0[ACC_ADDR_LENGTH-6..]].concat();

        //Uses unsafe because ``AccountAddress`` can not be created with invalid UTF-8 characters
        unsafe{String::from_utf8_unchecked(truncated)}
    }

    // Uses unsafe because ``AccountAddress`` can not be created with invalid UTF-8 characters
    // pub fn truncate(&self) -> String {
    //     let mut truncated = String::with_capacity(ACC_TRUNCATE_LEN);
    //     truncated.push_str(unsafe { std::str::from_utf8_unchecked(&self.0[0..4]) });
    //     truncated.push_str("...");
    //     truncated.push_str(unsafe {
    //         std::str::from_utf8_unchecked(&self.0[ACC_ADDR_LENGTH - 6..])
    //     });
    //     truncated
    // }
}

impl FromStr for AccountAddress {
    type Err = ParseAddrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(ParseAddrError::NonAsciiCharacter);
        }
        Ok(Self(s.as_bytes().try_into().map_err(|_| {
            ParseAddrError::InvalidLength {
                expected: ACC_ADDR_LENGTH,
                found: s.len(),
            }
        })?))
    }
}

impl TryFrom<&[u8]> for AccountAddress {
    type Error = ParseAddrError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // impl Regex check for exact characters and length at once
        if !value.is_ascii() {
            return Err(ParseAddrError::NonAsciiCharacter);
        }

        Ok(Self(value.try_into().map_err(|_| {
            ParseAddrError::InvalidLength {
                expected: ACC_ADDR_LENGTH,
                found: value.len(),
            }
        })?))
    }
}

impl ToString for AccountAddress {
    fn to_string(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.0.to_vec()) }
    }
}

impl Serialize for AccountAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("ResourceAddress", &self.0[..])
    }
}

impl<'de> Deserialize<'de> for AccountAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let slice: &[u8] = Deserialize::deserialize(deserializer)?;

        Ok(Self(slice.try_into().map_err(|_| {
            Error::custom(ParseAddrError::InvalidLength {
                expected: ACC_ADDR_LENGTH,
                found: slice.len(),
            })
        })?))
    }
}

impl rusqlite::types::FromSql for AccountAddress {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => Ok(Self(slice.try_into().map_err(|_| {
                rusqlite::types::FromSqlError::InvalidBlobSize {
                    expected_size: ACC_ADDR_LENGTH,
                    blob_size: slice.len(),
                }
            })?)),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for AccountAddress {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(&self.0),
        ))
    }
}
