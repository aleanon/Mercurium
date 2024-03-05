pub mod account_name;
pub mod settings;
pub mod transaction;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub use {
    account_name::AccountName,
    settings::{DepositRules, Settings},
    transaction::Transaction,
};

use super::{Fungibles, NonFungibles};

use super::AccountAddress;

#[derive(Debug, Clone)]
pub struct EntityAccount {
    pub id: usize,
    pub name: String,
    pub address: AccountAddress,
    pub fungibles: Fungibles,
    pub non_fungibles: Option<NonFungibles>,
    pub transactions: Option<BTreeSet<Transaction>>,
    pub settings: Settings,
}

impl EntityAccount {
    pub fn new(id: usize, name: String, address: AccountAddress) -> Self {
        Self {
            id,
            name,
            address,
            fungibles: Fungibles::new(),
            non_fungibles: None,
            transactions: None,
            settings: Settings::default(),
        }
    }

    // pub fn from_component(id: usize, component: Entity, name: String) -> Self {

    //     Self {
    //         id,
    //         name,
    //         address: component.address,
    //         fungibles: component.fungible_resources.into(),
    //         non_fungibles: Some(component.non_fungible_resources.into()),
    //         transactions: None,
    //         settings: Settings::default(),
    //     }
    // }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_address(&self) -> &AccountAddress {
        &self.address
    }

    pub fn deposit_rules(&self) -> &DepositRules {
        &self.settings.third_party_deposits
    }
}

impl PartialEq for EntityAccount {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
    }
}

impl Eq for EntityAccount {}

impl PartialOrd for EntityAccount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for EntityAccount {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct Time;
