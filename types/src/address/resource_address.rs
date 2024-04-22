use std::ffi::OsString;
use std::str::FromStr;

// use anyhow::Result;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use super::ParseAddrError;

const RESOURCE_ADDRESS_LEN: usize = 67;
const RES_ADDR_TRUNCATE_LEN: usize = 13;
// #[derive(Debug, Error)]
// pub enum ParseAddrError {
//     InvalidLength{
//         expected: usize,
//         found: usize,
//     },
//     NonAsciiCharacter,
// }

// impl std::fmt::Display for ParseAddrError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ParseAddrError::NonAsciiCharacter => write!(f, "Non ASCII character"),
//             ParseAddrError::InvalidLength { expected, found } => write!(f, "Invalid length, expected {}, found {}", expected, found),
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceAddress([u8; RESOURCE_ADDRESS_LEN]);

impl ResourceAddress {
    pub fn as_ref(&self) -> &[u8] {
        &self.0
    }

    // Uses unsafe because ``ResourceAddress`` can not be created with invalid UTF-8 characters
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }

    // Uses unsafe because ``ResourceAddress`` can not be created with invalid UTF-8 characters
    pub fn truncate(&self) -> String {
        let mut truncated = String::with_capacity(RES_ADDR_TRUNCATE_LEN);
        truncated.push_str(unsafe { std::str::from_utf8_unchecked(&self.0[0..4]) });
        truncated.push_str("...");
        truncated.push_str(unsafe {
            std::str::from_utf8_unchecked(&self.0[RESOURCE_ADDRESS_LEN - 6..RESOURCE_ADDRESS_LEN])
        });
        truncated
    }
}

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = ParseAddrError;
    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if !value.is_ascii() {
            return Err(ParseAddrError::NonAsciiCharacter);
        }

        Ok(Self(value.try_into()?))
    }
}

impl TryFrom<&OsString> for ResourceAddress {
    type Error = ParseAddrError;

    fn try_from(value: &OsString) -> Result<Self, Self::Error> {
        ResourceAddress::try_from(value.as_encoded_bytes())
    }
}

impl FromStr for ResourceAddress {
    type Err = ParseAddrError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        //switch to regex and check for specific constraints
        if !s.is_ascii() {
            return Err(ParseAddrError::NonAsciiCharacter);
        }

        Ok(Self(s.as_bytes().try_into()?))
    }
}

impl ToString for ResourceAddress {
    fn to_string(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.0.to_vec()) }
    }
}

impl Serialize for ResourceAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("ResourceAddress", &self.0[..])
    }
}

impl<'de> Deserialize<'de> for ResourceAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let slice: &[u8] = Deserialize::deserialize(deserializer)?;

        Ok(Self(slice.try_into().map_err(|err| Error::custom(err))?))
    }
}

impl rusqlite::types::FromSql for ResourceAddress {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => Ok(Self(slice.try_into().map_err(|_| {
                rusqlite::types::FromSqlError::InvalidBlobSize {
                    expected_size: RESOURCE_ADDRESS_LEN,
                    blob_size: slice.len(),
                }
            })?)),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for ResourceAddress {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(self.0.as_ref()),
        ))
    }
}
