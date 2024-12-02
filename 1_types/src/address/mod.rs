pub(crate) mod account_address;
pub(crate) mod address_validator;
pub(crate) mod resource_address;
pub(crate) mod transaction_address;

use std::str::FromStr;
use thiserror::Error;

pub use account_address::*;
pub use address_validator::*;
pub use resource_address::*;
pub use transaction_address::*;

use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable, Network};

pub enum AddressType {
    Account,
    Resource,
    Component,
    Transaction,
}

#[derive(Debug, Error)]
pub enum AddressError {
    #[error("Invalid address length")]
    InvalidLength,
    #[error("Invalid address prefix")]
    InvalidPrefix,
    #[error("Invalid utf8 in address")]
    InvalidUTF8,
    #[error("Not a valid radix dlt address")]
    InvalidAddress,
}

const fn get_address_type_prefix(address_type: AddressType) -> &'static str {
    match address_type {
        AddressType::Account => "account_",
        AddressType::Resource => "resource_",
        AddressType::Component => "component_",
        AddressType::Transaction => "txid_",
    }
}

/// Gets the length of address without type prefix and network prefix
const fn get_address_length(address_type: AddressType) -> usize {
    match address_type {
        AddressType::Account => 54,
        AddressType::Resource => 54,
        AddressType::Component => 54,
        AddressType::Transaction => 58,
    }
}

pub trait Address: Sized + FromStr {
    const ADDRESS_TYPE: AddressType;
    const MAINNET_REGEX_PATTERN: &'static str;
    const STOKENET_REGEX_PATTERN: &'static str;
    const ADDRESS_TYPE_PREFIX: &'static str = get_address_type_prefix(Self::ADDRESS_TYPE);
    const MAINNET_PREFIX: &'static str = Network::MAINNET_PREFIX;
    const STOKENET_PREFIX: &'static str = Network::STOKENET_PREFIX;
    const MAINNET_LENGTH: usize =
        Self::ADDRESS_TYPE_PREFIX_LENGTH + Network::MAINNET_PREFIX.len() + Self::ADDRESS_LENGTH;
    const STOKENET_LENGTH: usize =
        Self::ADDRESS_TYPE_PREFIX_LENGTH + Network::STOKENET_PREFIX.len() + Self::ADDRESS_LENGTH;
    const ADDRESS_TYPE_PREFIX_LENGTH: usize = Self::ADDRESS_TYPE_PREFIX.len();
    const ADDRESS_LENGTH: usize = get_address_length(Self::ADDRESS_TYPE);
    const MAINNET_ADDRESS_START_INDEX: usize = Self::MAINNET_LENGTH - Self::ADDRESS_LENGTH;
    const STOKENET_ADDRESS_START_INDEX: usize = Self::STOKENET_LENGTH - Self::ADDRESS_LENGTH;
    const CHECKSUM_LENGTH: usize = 6;
    const CHECKSUM_DOUBLE_LENGTH: usize = Self::CHECKSUM_LENGTH * 2;
    const MAINNET_CHECKSUM_START_INDEX: usize = Self::MAINNET_LENGTH - Self::CHECKSUM_LENGTH;
    const STOKENET_CHECKSUM_START_INDEX: usize = Self::STOKENET_LENGTH - Self::CHECKSUM_LENGTH;
    const MAINNET_CHECKSUM_DOUBLE_START_INDEX: usize =
        Self::MAINNET_LENGTH - Self::CHECKSUM_DOUBLE_LENGTH;
    const STOKENET_CHECKSUM_DOUBLE_START_INDEX: usize =
        Self::STOKENET_LENGTH - Self::CHECKSUM_DOUBLE_LENGTH;
    const TRUNCATE_PREFIX_LEN: usize = 4;
    const TRUNCATE_LONG_PREFIX_LEN: usize = 12;
    const TRUNCATE_DOTS: &'static str = "...";

    fn as_bytes(&self) -> &[u8];

    fn checksum_start_index(&self) -> usize;

    fn checksum_double_start_index(&self) -> usize;

    fn address_start_index(&self) -> usize;

    fn is_valid_address(network: Network, address: &str) -> bool;

    fn network(&self) -> Network;

    fn as_str(&self) -> &str {
        std::str::from_utf8(self.as_bytes())
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn checksum_slice(&self) -> &[u8] {
        &self.as_bytes()[self.checksum_start_index()..]
    }

    fn checksum_as_str(&self) -> &str {
        std::str::from_utf8(self.checksum_slice())
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn checksum_double_slice(&self) -> &[u8] {
        &self.as_bytes()[self.checksum_double_start_index()..]
    }

    fn checksum_double_as_str(&self) -> &str {
        std::str::from_utf8(self.checksum_double_slice())
            .unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn address_excluding_prefix(&self) -> &[u8] {
        &self.as_bytes()[self.address_start_index()..]
    }

    fn truncate(&self) -> String {
        let slice = self.as_bytes();
        let truncated = [
            &slice[..Self::TRUNCATE_PREFIX_LEN],
            Self::TRUNCATE_DOTS.as_bytes(),
            &slice[self.checksum_start_index()..],
        ]
        .concat();

        String::from_utf8(truncated).unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn truncate_long(&self) -> String {
        let slice = self.as_bytes();
        let truncated = [
            &slice[..Self::TRUNCATE_LONG_PREFIX_LEN],
            Self::TRUNCATE_DOTS.as_bytes(),
            &slice[self.checksum_start_index()..],
        ]
        .concat();

        String::from_utf8(truncated).unwrap_unreachable(debug_info!("Invalid UTF-8 in address"))
    }

    fn address_type(&self) -> AddressType {
        Self::ADDRESS_TYPE
    }
}
