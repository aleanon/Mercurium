use crate::unwrap_unreachable::UnwrapUnreachable;
use crate::{debug_info, Network};

use super::{Address, AddressError, AddressType};
use async_sqlite::rusqlite;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::str::FromStr;

static MAINNET_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(TransactionAddress::MAINNET_REGEX_PATTERN).unwrap());

static STOKENET_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(TransactionAddress::STOKENET_REGEX_PATTERN).unwrap());

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransactionAddress {
    Mainnet([u8; Self::MAINNET_LENGTH]),
    Stokenet([u8; Self::STOKENET_LENGTH]),
}

impl Address for TransactionAddress {
    const ADDRESS_TYPE: AddressType = AddressType::Transaction;
    const MAINNET_REGEX_PATTERN: &'static str = const_format::formatcp!(
        "^{}{}[a-z0-9]{{{}}}$",
        TransactionAddress::ADDRESS_TYPE_PREFIX,
        TransactionAddress::MAINNET_PREFIX,
        TransactionAddress::ADDRESS_LENGTH
    );
    const STOKENET_REGEX_PATTERN: &'static str = const_format::formatcp!(
        "^{}{}[a-z0-9]{{{}}}$",
        TransactionAddress::ADDRESS_TYPE_PREFIX,
        TransactionAddress::STOKENET_PREFIX,
        TransactionAddress::ADDRESS_LENGTH
    );

    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Mainnet(bytes) => bytes,
            Self::Stokenet(bytes) => bytes,
        }
    }

    fn checksum_start_index(&self) -> usize {
        match self {
            Self::Mainnet(_) => Self::MAINNET_CHECKSUM_START_INDEX,
            Self::Stokenet(_) => Self::STOKENET_CHECKSUM_START_INDEX,
        }
    }

    fn checksum_double_start_index(&self) -> usize {
        match self {
            Self::Mainnet(_) => Self::MAINNET_CHECKSUM_DOUBLE_START_INDEX,
            Self::Stokenet(_) => Self::STOKENET_CHECKSUM_DOUBLE_START_INDEX,
        }
    }

    fn address_start_index(&self) -> usize {
        match self {
            Self::Mainnet(_) => Self::MAINNET_ADDRESS_START_INDEX,
            Self::Stokenet(_) => Self::STOKENET_ADDRESS_START_INDEX,
        }
    }

    fn is_valid_address(network: Network, address: &str) -> bool {
        match network {
            Network::Mainnet => MAINNET_REGEX.is_match(address),
            Network::Stokenet => STOKENET_REGEX.is_match(address),
        }
    }

    fn network(&self) -> Network {
        match self {
            Self::Mainnet(_) => Network::Mainnet,
            Self::Stokenet(_) => Network::Stokenet,
        }
    }
}

impl TransactionAddress {
    #[cfg(test)]
    pub fn empty(network: Network) -> Self {
        match network {
            Network::Mainnet => TransactionAddress::Mainnet([0; Self::MAINNET_LENGTH]),
            Network::Stokenet => TransactionAddress::Stokenet([0; Self::STOKENET_LENGTH]),
        }
    }
}

impl FromStr for TransactionAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if MAINNET_REGEX.is_match(s) {
            return Ok(Self::Mainnet(
                s.as_bytes()
                    .try_into()
                    .map_err(|_| AddressError::InvalidLength)?,
            ));
        }
        if STOKENET_REGEX.is_match(s) {
            return Ok(Self::Stokenet(
                s.as_bytes()
                    .try_into()
                    .map_err(|_| AddressError::InvalidLength)?,
            ));
        }
        Err(AddressError::InvalidAddress)
    }
}

impl ToString for TransactionAddress {
    fn to_string(&self) -> String {
        String::from_utf8(self.as_bytes().to_vec())
            .unwrap_unreachable(debug_info!("Invalid Utf8 in AccountAddress"))
    }
}

impl TryFrom<&[u8]> for TransactionAddress {
    type Error = AddressError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(value).map_err(|_| AddressError::InvalidUTF8)?;
        TransactionAddress::from_str(s)
    }
}

impl Serialize for TransactionAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TransactionAddress::Mainnet(bytes) => serializer.serialize_newtype_variant(
                "TransactionAddress",
                0,
                "Mainnet",
                bytes.as_slice(),
            ),
            TransactionAddress::Stokenet(bytes) => serializer.serialize_newtype_variant(
                "TransactionAddress",
                1,
                "Stokenet",
                bytes.as_slice(),
            ),
        }
    }
}

impl<'de> Deserialize<'de> for TransactionAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let slice: &[u8] = Deserialize::deserialize(deserializer)?;

        Ok(Self::try_from(slice).map_err(|err| Error::custom(err))?)
    }
}

impl rusqlite::types::FromSql for TransactionAddress {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => Ok(
                Self::try_from(slice).map_err(|_| rusqlite::types::FromSqlError::InvalidType)?
            ),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for TransactionAddress {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(&self.as_bytes()),
        ))
    }
}
