pub mod account_address;
pub mod component_address;
pub mod internal_vault_address;
pub mod resource_address;
pub mod transaction_address;

use std::{array::TryFromSliceError, fmt::Display};
use thiserror::Error;

pub use account_address::AccountAddress;
pub use resource_address::ResourceAddress;

use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable, Network};

#[derive(Debug, Error)]
pub enum ParseAddrError {
    #[error("Non ASCII character")]
    NonAsciiCharacter,
    #[error("{0}")]
    InvalidLength(#[from] TryFromSliceError),
    #[error("Invalid network prefix")]
    InvalidPrefix,
}

pub enum Address {
    AccountAddress(AccountAddress),
    ResourceAddress(ResourceAddress),
    InternalVaultAddress,
    ComponentAddress,
}

impl Address {
    pub const TRUNCATE_DOT_COUNT: usize = 3;
    pub const TRUNCATE_PREFIX_LEN: usize = 4;
    pub const TRUNCATE_LONG_PREFIX_LEN: usize = 12;
    pub const TRUNCATE_DOTS: &'static str = "...";
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::AccountAddress(address) => write!(f, "{}", address.as_str()),
            Address::ResourceAddress(address) => write!(f, "{}", address.as_str()),
            _ => todo!(),
        }
    }
}

impl From<AccountAddress> for Address {
    fn from(value: AccountAddress) -> Self {
        Self::AccountAddress(value)
    }
}

impl From<ResourceAddress> for Address {
    fn from(value: ResourceAddress) -> Self {
        Self::ResourceAddress(value)
    }
}

pub enum AddressType {
    Account,
    Resource,
    InternalVault,
    Component,
    Transaction,
}

pub enum AddressError {
    InvalidLength,
    InvalidPrefix,
    InvalidUTF8,
    InvalidAddress,
}

const fn get_network_prefix(network: Network) -> &'static str {
    match network {
        Network::Mainnet => Network::MAINNET_PREFIX_STR,
        Network::Stokenet => Network::STOKENET_PREFIX_STR,
    }
}

const fn get_address_type_prefix(address_type: AddressType) -> &'static str {
    match address_type {
        AddressType::Account => "account_",
        AddressType::Resource => "resource_",
        AddressType::InternalVault => "internal_vault_",
        AddressType::Component => "component_",
        AddressType::Transaction => "txid_",
    }
}

const fn get_address_length(address_type: AddressType) -> usize {
    match address_type {
        AddressType::Account => 54,
        AddressType::Resource => 54,
        AddressType::InternalVault => 54,
        AddressType::Component => 54,
        AddressType::Transaction => 58,
    }
}

const fn evaluate_prefix_len(str: &'static str) -> usize {
    str.len()
}

trait AddressTrait: Sized {
    const NETWORK: Network;
    const ADDRESS_TYPE: AddressType;
    const REGEX_PATTERN: &'static str;
    const ADDRESS_TYPE_PREFIX: &'static str = get_address_type_prefix(Self::ADDRESS_TYPE);
    const NETWORK_PREFIX: &'static str = get_network_prefix(Self::NETWORK);
    const LENGTH: usize =
        Self::ADDRESS_TYPE_PREFIX_LENGTH + Self::NETWORK_PREFIX_LENGTH + Self::ADDRESS_LENGTH;
    const ADDRESS_TYPE_PREFIX_LENGTH: usize = evaluate_prefix_len(Self::ADDRESS_TYPE_PREFIX);
    const NETWORK_PREFIX_LENGTH: usize = evaluate_prefix_len(Self::NETWORK_PREFIX);
    const ADDRESS_LENGTH: usize = get_address_length(Self::ADDRESS_TYPE);
    const ADDRESS_START_INDEX: usize = Self::LENGTH - Self::ADDRESS_LENGTH;
    const CHECKSUM_LENGTH: usize = 6;
    const CHECKSUM_START_INDEX: usize = Self::LENGTH - Self::CHECKSUM_LENGTH;
    const CHECKSUM_DOUBLE_START_INDEX: usize = Self::CHECKSUM_START_INDEX - Self::CHECKSUM_LENGTH;
    const TRUNCATE_PREFIX_LEN: usize = 4;
    const TRUNCATE_LONG_PREFIX_LEN: usize = 12;
    const TRUNCATE_DOTS: &'static str = "...";

    fn as_bytes(&self) -> &[u8];

    fn from_str(s: &str) -> Result<Self, AddressError>;

    fn from_bytes_without_prefixes(address: &[u8]) -> Result<Self, AddressError>;

    fn is_valid_address(address: &str) -> bool;

    fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes())
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn checksum(&self) -> &[u8] {
        &self.as_bytes()[Self::CHECKSUM_START_INDEX..]
    }

    fn checksum_as_str(&self) -> &str {
        std::str::from_utf8(self.checksum())
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn checksum_plus_equal_part_of_address(&self) -> &[u8] {
        &self.as_bytes()[Self::CHECKSUM_DOUBLE_START_INDEX..]
    }

    fn address_excluding_prefix(&self) -> &[u8] {
        &self.as_bytes()[Self::ADDRESS_START_INDEX..]
    }

    fn truncate(&self) -> String {
        let slice = self.as_bytes();
        let truncated = [
            &slice[..Self::TRUNCATE_PREFIX_LEN],
            Self::TRUNCATE_DOTS.as_bytes(),
            &slice[Self::CHECKSUM_START_INDEX..],
        ]
        .concat();

        String::from_utf8(truncated).unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn truncate_long(&self) -> String {
        let slice = self.as_bytes();
        let truncated = [
            &slice[..Self::TRUNCATE_LONG_PREFIX_LEN],
            Self::TRUNCATE_DOTS.as_bytes(),
            &slice[Self::CHECKSUM_START_INDEX..],
        ]
        .concat();

        String::from_utf8(truncated).unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn address_type(&self) -> AddressType {
        Self::ADDRESS_TYPE
    }

    fn network(&self) -> Network {
        Self::NETWORK
    }
}
