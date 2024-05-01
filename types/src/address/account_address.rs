use super::ParseAddrError;
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
pub struct AccountAddress([u8; Self::LENGTH]);

impl AccountAddress {
    pub const LENGTH: usize = 66;
    pub const CHECKSUM_LEN: usize = 6;
    const TRUNCATED_LEN: usize = 13;
    const TRUNCATED_LONG_LEN: usize = 21;
    const PREFIX: &'static str = "account_";

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
        self.0 == [b'0'; ACC_ADDR_LENGTH]
    }

    pub fn truncate(&self) -> String {
        let truncated = [&self.0[..4], &[b'.'; 3], &self.0[ACC_ADDR_LENGTH - 6..]].concat();

        //Uses unchecked because ``AccountAddress`` can not be created with invalid UTF-8 characters
        unsafe { String::from_utf8_unchecked(truncated) }
    }

    pub fn truncate_str(&self) -> &str {
        let truncated = [&self.0[..4], &[b'.'; 3], &self.0[ACC_ADDR_LENGTH - 6..]];

        //Uses unchecked because ``AccountAddress`` can not be created with invalid UTF-8 characters
        unsafe {
            let slice =
                std::slice::from_raw_parts(truncated.as_ptr() as *const u8, Self::TRUNCATED_LEN);
            std::str::from_utf8_unchecked(slice)
        }
    }

    pub fn truncate_long(&self) -> String {
        let truncated = [&self.0[..12], &[b'.'; 3], &self.0[ACC_ADDR_LENGTH - 6..]].concat();

        //Uses unchecked because ``AccountAddress`` can not be created with invalid UTF-8 characters
        unsafe { String::from_utf8_unchecked(truncated) }
    }

    pub fn truncate_long_str(&self) -> &str {
        let truncated = [&self.0[..12], &[b'.'; 3], &self.0[ACC_ADDR_LENGTH - 6..]];

        //Uses unchecked because ``AccountAddress`` can not be created with invalid UTF-8 characters
        unsafe {
            let slice = std::slice::from_raw_parts(
                truncated.as_ptr() as *const u8,
                Self::TRUNCATED_LONG_LEN,
            );
            std::str::from_utf8_unchecked(slice)
        }
    }

    pub fn checksum(&self) -> [u8; 6] {
        self.0[ACC_ADDR_LENGTH - 6..]
            .try_into()
            .unwrap_or_else(|_| unreachable!())
    }

    pub fn checksum_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0[ACC_ADDR_LENGTH - 6..]) }
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
        Ok(Self(s.as_bytes().try_into()?))
    }
}

impl TryFrom<&[u8]> for AccountAddress {
    type Error = ParseAddrError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // impl Regex check for exact characters and length at once
        if !value.is_ascii() {
            return Err(ParseAddrError::NonAsciiCharacter);
        }

        Ok(Self(value.try_into()?))
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

        Ok(Self(slice.try_into().map_err(|err| Error::custom(err))?))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_str() {
        let addr = AccountAddress::from_str(
            "account_rdx12ymqrlezhreuknut5x5ucq30he638pqu9wum7nuxl65z9pjdt2a5ax",
        )
        .unwrap();

        let truncated = addr.truncate_str();
        assert!(truncated == "acco...t2a5ax");

        let truncated_long = addr.truncate_long_str();
        assert!(truncated_long == "account_rdx1...t2a5ax");
    }

    #[test]
    fn test_truncate_long_str() {}
}
