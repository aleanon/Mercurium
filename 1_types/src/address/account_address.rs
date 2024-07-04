use crate::unwrap_unreachable::UnwrapUnreachable;
use crate::{debug_info, Address, Network};

use super::{AddressError, AddressTrait, AddressType, ParseAddrError};
use once_cell::sync::Lazy;
use regex::Regex;
use rusqlite::types::FromSql;
use rusqlite::ToSql;
use scrypto::network::NetworkDefinition;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::str::FromStr;

trait AccountAddressTrait: AddressTrait {}

pub struct MainnetAccountAddress([u8; Self::LENGTH]);

impl AccountAddressTrait for MainnetAccountAddress {}

impl AddressTrait for MainnetAccountAddress {
    const ADDRESS_TYPE: AddressType = AddressType::Account;
    const NETWORK: Network = Network::Mainnet;
    const REGEX_PATTERN: &'static str = const_format::formatcp!(
        "^{}{}[a-z0-9]{{{}}}$",
        MainnetAccountAddress::ADDRESS_TYPE_PREFIX,
        MainnetAccountAddress::NETWORK_PREFIX,
        MainnetAccountAddress::ADDRESS_LENGTH
    );

    fn from_str(s: &str) -> Result<Self, AddressError> {
        if Self::is_valid_address(s) {
            let mut array = [0u8; Self::LENGTH];
            array.copy_from_slice(s.as_bytes());

            Ok(Self(array))
        } else {
            Err(AddressError::InvalidAddress)
        }
    }

    fn from_bytes_without_prefixes(address: &[u8]) -> Result<Self, AddressError> {
        if address.len() != Self::ADDRESS_LENGTH {
            return Err(AddressError::InvalidLength);
        }

        if !address.is_ascii() {
            return Err(AddressError::InvalidUTF8);
        }

        let mut array = [0u8; Self::LENGTH];
        array[..Self::ADDRESS_TYPE_PREFIX_LENGTH]
            .copy_from_slice(Self::ADDRESS_TYPE_PREFIX.as_bytes());
        array[Self::ADDRESS_TYPE_PREFIX_LENGTH..Self::ADDRESS_START_INDEX]
            .copy_from_slice(Self::NETWORK_PREFIX.as_bytes());
        array[Self::ADDRESS_START_INDEX..].copy_from_slice(address);

        Ok(Self(array))
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn is_valid_address(address: &str) -> bool {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(MainnetAccountAddress::REGEX_PATTERN).unwrap());
        REGEX.is_match(address)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountAddress([u8; Self::LENGTH]);

impl AccountAddress {
    const PREFIX_LENGTH: usize = 8;
    pub const LENGTH: usize = 66;
    pub const CHECKSUM_LENGTH: usize = 6;
    const CHECKSUM_START_INDEX: usize = Self::LENGTH - Self::CHECKSUM_LENGTH;
    const CHECKSUM_DOUBLE_START_INDEX: usize = Self::CHECKSUM_START_INDEX - Self::CHECKSUM_LENGTH;
    const TRUNCATE_PREFIX_LEN: usize = 4;
    const TRUNCATE_LONG_PREFIX_LEN: usize = 12;
    const TRUNCATE_LENGTH: usize = Self::TRUNCATE_PREFIX_LEN + 3 + Self::CHECKSUM_LENGTH;
    const PREFIX: &'static str = "account_";

    pub fn empty() -> Self {
        Self([b'0'; Self::LENGTH])
    }

    pub fn as_ref(&self) -> &[u8] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0)
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in AccountAddress"))
    }

    pub fn is_empty(&self) -> bool {
        self.0 == [b'0'; Self::LENGTH]
    }

    pub fn truncate(&self) -> String {
        let truncated = [
            &self.0[..Self::TRUNCATE_PREFIX_LEN],
            Address::TRUNCATE_DOTS.as_bytes(),
            &self.0[Self::CHECKSUM_START_INDEX..],
        ]
        .concat();

        String::from_utf8(truncated)
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in AccountAddress"))
    }

    pub fn truncate_as_array(&self) -> [u8; Self::TRUNCATE_LENGTH] {
        let mut truncated = [b'.'; Self::TRUNCATE_LENGTH];

        truncated[..Self::TRUNCATE_PREFIX_LEN]
            .copy_from_slice(&self.0[..Self::TRUNCATE_PREFIX_LEN]);
        truncated[Self::TRUNCATE_LENGTH - Self::PREFIX_LENGTH..]
            .copy_from_slice(&self.0[Self::CHECKSUM_START_INDEX..]);

        truncated
    }

    pub fn truncate_long(&self) -> String {
        let truncated = [
            &self.0[..Self::TRUNCATE_LONG_PREFIX_LEN],
            Address::TRUNCATE_DOTS.as_bytes(),
            &self.0[Self::CHECKSUM_START_INDEX..],
        ]
        .concat();

        String::from_utf8(truncated)
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in AccountAddress"))
    }

    pub fn checksum(&self) -> [u8; Self::CHECKSUM_LENGTH] {
        self.0[Self::CHECKSUM_START_INDEX..]
            .try_into()
            .unwrap_unreachable(debug_info!("Invalid Checksum Length"))
    }

    pub fn checksum_slice(&self) -> &[u8] {
        &self.0[Self::CHECKSUM_START_INDEX..]
    }

    pub fn checksum_str(&self) -> &str {
        std::str::from_utf8(&self.0[Self::CHECKSUM_START_INDEX..])
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in AccountAddress"))
    }

    pub fn checksum_double(&self) -> &[u8] {
        &self.0[Self::CHECKSUM_DOUBLE_START_INDEX..]
    }
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
        String::from_utf8(self.0.to_vec())
            .unwrap_unreachable(debug_info!("Invalid Utf8 in AccountAddress"))
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
                    expected_size: Self::LENGTH,
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
