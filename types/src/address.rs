pub mod resource_address;
pub mod account_address;
pub mod internal_vault_address;
pub mod component_address;

use std::fmt::Display;
use thiserror::Error;

pub use resource_address::ResourceAddress;
pub use account_address::AccountAddress;


#[derive(Debug, Error)]
pub enum ParseAddrError {
    #[error("Non ASCII character")]
    NonAsciiCharacter,
    #[error("Invalid length, expected: {expected}, found: {found}")]
    InvalidLength {
        expected: usize,
        found: usize,
    }
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
            _ => todo!()
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
