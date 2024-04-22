pub mod account_address;
pub mod component_address;
pub mod internal_vault_address;
pub mod resource_address;

use std::{array::TryFromSliceError, fmt::Display};
use thiserror::Error;

pub use account_address::AccountAddress;
pub use resource_address::ResourceAddress;

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
