use super::AddressValidator;
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
    Lazy::new(|| Regex::new(AccountAddress::MAINNET_REGEX_PATTERN).unwrap());

static STOKENET_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(AccountAddress::STOKENET_REGEX_PATTERN).unwrap());

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AccountAddress {
    Mainnet([u8; Self::MAINNET_LENGTH]),
    Stokenet([u8; Self::STOKENET_LENGTH]),
}

impl Address for AccountAddress {
    const ADDRESS_TYPE: AddressType = AddressType::Account;
    const MAINNET_REGEX_PATTERN: &'static str = const_format::formatcp!(
        "^{}{}[a-z0-9]{{{}}}$",
        AccountAddress::ADDRESS_TYPE_PREFIX,
        AccountAddress::MAINNET_PREFIX,
        AccountAddress::ADDRESS_LENGTH
    );
    const STOKENET_REGEX_PATTERN: &'static str = const_format::formatcp!(
        "^{}{}[a-z0-9]{{{}}}$",
        AccountAddress::ADDRESS_TYPE_PREFIX,
        AccountAddress::STOKENET_PREFIX,
        AccountAddress::ADDRESS_LENGTH
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
        AddressValidator::is_valid_account(network, address)
    }

    fn network(&self) -> Network {
        match self {
            Self::Mainnet(_) => Network::Mainnet,
            Self::Stokenet(_) => Network::Stokenet,
        }
    }
}

impl AccountAddress {
    #[cfg(test)]
    pub fn empty(network: Network) -> Self {
        match network {
            Network::Mainnet => AccountAddress::Mainnet([0; Self::MAINNET_LENGTH]),
            Network::Stokenet => AccountAddress::Stokenet([0; Self::STOKENET_LENGTH]),
        }
    }
}

impl FromStr for AccountAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if Self::is_valid_address(Network::Mainnet, s) {
            return Ok(Self::Mainnet(
                s.as_bytes()
                    .try_into()
                    .map_err(|_| AddressError::InvalidLength)?,
            ));
        }
        if Self::is_valid_address(Network::Stokenet, s) {
            return Ok(Self::Stokenet(
                s.as_bytes()
                    .try_into()
                    .map_err(|_| AddressError::InvalidLength)?,
            ));
        }
        Err(AddressError::InvalidAddress)
    }
}

impl ToString for AccountAddress {
    fn to_string(&self) -> String {
        String::from_utf8(self.as_bytes().to_vec())
            .unwrap_unreachable(debug_info!("Invalid Utf8 in AccountAddress"))
    }
}

impl TryFrom<&[u8]> for AccountAddress {
    type Error = AddressError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(value).map_err(|_| AddressError::InvalidUTF8)?;
        AccountAddress::from_str(s)
    }
}

impl Serialize for AccountAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AccountAddress::Mainnet(bytes) => serializer.serialize_newtype_variant(
                "AccountAddress",
                0,
                "Mainnet",
                bytes.as_slice(),
            ),
            AccountAddress::Stokenet(bytes) => serializer.serialize_newtype_variant(
                "AccountAddress",
                1,
                "Stokenet",
                bytes.as_slice(),
            ),
        }
    }
}

impl<'de> Deserialize<'de> for AccountAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let slice: &[u8] = Deserialize::deserialize(deserializer)?;

        Ok(Self::try_from(slice).map_err(|err| Error::custom(err))?)
    }
}

impl rusqlite::types::FromSql for AccountAddress {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(slice) => Ok(
                Self::try_from(slice).map_err(|_| rusqlite::types::FromSqlError::InvalidType)?
            ),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for AccountAddress {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(&self.as_bytes()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
