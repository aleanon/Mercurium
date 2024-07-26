use scrypto::crypto::Ed25519PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

use super::{
    address::{AccountAddress, Address, ResourceAddress},
    Network,
};
use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable};

pub struct AccountCollection(pub Vec<Account>);

#[derive(Debug, Clone, Eq)]
pub struct Account {
    pub address: AccountAddress,
    pub id: usize,
    pub name: String,
    pub network: Network,
    //Stored the derivation path as bytes for serialization in the database
    pub derivation_path: [u8; 24],
    pub public_key: Ed25519PublicKey,
    pub hidden: bool,
    pub settings: Settings,
    /// Last updated refers to the Ledger state version
    pub balances_last_updated: Option<i64>,
    pub transactions_last_updated: Option<i64>,
}

impl Account {
    pub fn new(
        id: usize,
        name: String,
        network: Network,
        derivation_path: [u32; 6],
        address: AccountAddress,
        public_key: Ed25519PublicKey,
    ) -> Self {
        let mut path = [0u8; 24];

        for i in 0..derivation_path.len() {
            let bytes = derivation_path[i].to_be_bytes();
            path[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }

        Self {
            id,
            name,
            network,
            derivation_path: path,
            address,
            public_key,
            hidden: false,
            settings: Settings::default(),
            balances_last_updated: None,
            transactions_last_updated: None,
        }
    }

    #[cfg(test)]
    pub fn none(network: Network) -> Self {
        let pub_key = [0u8; Ed25519PublicKey::LENGTH];
        Self {
            id: 0,
            name: "No Account".to_owned(),
            network,
            derivation_path: [0u8; 24],
            address: AccountAddress::empty(network),
            public_key: Ed25519PublicKey::try_from(pub_key.as_slice())
                .expect("Can not create public key from slice, module Account"),
            hidden: false,
            settings: Settings::default(),
            balances_last_updated: None,
            transactions_last_updated: None,
        }
    }

    //Todo: implement test for this
    pub fn derivation_path(&self) -> [u32; 6] {
        let mut path = [0u32; 6];

        for i in 0..path.len() {
            let bytes = self.derivation_path[i * 4..i * 4 + 4]
                .try_into()
                .unwrap_unreachable(debug_info!("Failed to convert derivation path from bytes"));
            path[i] = u32::from_be_bytes(bytes);
        }

        path
    }

    pub fn derivation_index(&self) -> u32 {
        let bytes = self.derivation_path[20..]
            .try_into()
            .unwrap_unreachable(debug_info!("Failed to construct array from slice"));
        u32::from_be_bytes(bytes)
    }
}

impl FromIterator<Account> for HashMap<AccountAddress, Account> {
    fn from_iter<T: IntoIterator<Item = Account>>(iter: T) -> Self {
        iter.into_iter()
            .map(|account| (account.address.clone(), account))
            .collect()
    }
}

impl ToString for Account {
    fn to_string(&self) -> String {
        format!("{}:    {}", self.name, self.address.truncate_long())
    }
}

impl ToString for &Account {
    fn to_string(&self) -> String {
        format!("{}:    {}", self.name, self.address.truncate_long())
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
    }
}

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for Account {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub(super) third_party_deposits: DepositRules,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            third_party_deposits: DepositRules::default(),
        }
    }
}

impl rusqlite::types::FromSql for Settings {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(blob) => Ok(serde_json::from_slice(blob)
                .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for Settings {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob(
                serde_json::to_vec(self)
                    .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?,
            ),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepositRules {
    accept_deposits: AcceptDeposits,
    allow_specific: Option<BTreeSet<ResourceAddress>>,
    deny_specific: Option<BTreeSet<ResourceAddress>>,
    allow_depositors: Option<BTreeSet<AccountAddress>>,
}

impl DepositRules {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accept_deposits(&mut self, rule: AcceptDeposits) {
        self.accept_deposits = rule
    }

    pub fn add_asset_allow(&mut self, resource_address: ResourceAddress) {
        match self.allow_specific {
            Some(ref mut tree) => tree.insert(resource_address),
            None => self
                .allow_specific
                .insert(BTreeSet::new())
                .insert(resource_address),
        };
    }

    pub fn add_asset_deny(&mut self, resource_address: ResourceAddress) {
        match self.deny_specific {
            Some(ref mut tree) => tree.insert(resource_address),
            None => self
                .deny_specific
                .insert(BTreeSet::new())
                .insert(resource_address),
        };
    }

    pub fn remove_asset_allow(&mut self, resource_address: ResourceAddress) {
        if let Some(ref mut tree) = self.allow_specific {
            tree.remove(&resource_address);
        }
    }

    pub fn remove_asset_deny(&mut self, resource_address: ResourceAddress) {
        if let Some(ref mut tree) = self.deny_specific {
            tree.remove(&resource_address);
        }
    }

    pub fn add_depositor(&mut self, depositor_address: AccountAddress) {
        match self.allow_depositors {
            Some(ref mut tree) => tree.insert(depositor_address),
            None => self
                .allow_depositors
                .insert(BTreeSet::new())
                .insert(depositor_address),
        };
    }

    pub fn remove_depositor(&mut self, depositor_address: AccountAddress) {
        if let Some(ref mut tree) = self.allow_depositors {
            tree.remove(&depositor_address);
            if tree.len() == 0 {
                self.allow_depositors = None
            };
        }
    }

    pub fn show_rules(
        &self,
    ) -> (
        &AcceptDeposits,
        &Option<BTreeSet<ResourceAddress>>,
        &Option<BTreeSet<ResourceAddress>>,
        &Option<BTreeSet<AccountAddress>>,
    ) {
        (
            &self.accept_deposits,
            &self.allow_specific,
            &self.deny_specific,
            &self.allow_depositors,
        )
    }
}

impl Default for DepositRules {
    fn default() -> Self {
        Self {
            accept_deposits: AcceptDeposits::All,
            allow_specific: None,
            deny_specific: None,
            allow_depositors: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum AcceptDeposits {
    #[default]
    All,
    Known,
    DenyAll,
}

impl std::fmt::Display for AcceptDeposits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcceptDeposits::All => write!(f, "Allow All"),
            AcceptDeposits::DenyAll => write!(f, "Deny All"),
            AcceptDeposits::Known => write!(f, "Allow Known"),
        }
    }
}

#[cfg(test)]
mod test {
    use scrypto::crypto::Ed25519PublicKey;

    use super::AccountAddress;

    use super::Account;

    use rand;

    #[test]
    fn test_derivation_path() {
        let derivation_path: [u32; 6] = [3244, 1022, 1, 525, 1460, 1];

        let public_key = Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]);

        let account = Account::new(
            0,
            "test".to_owned(),
            super::Network::Mainnet,
            derivation_path.clone(),
            AccountAddress::empty(super::Network::Mainnet),
            public_key,
        );

        let reconstructed_derivation_path = account.derivation_path();

        assert_eq!(derivation_path, reconstructed_derivation_path);
    }

    #[test]
    fn test_derivation_path_random() {
        for _ in 0..1000 {
            let derivation_path: [u32; 6] = [
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ];

            let public_key = Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]);

            let account = Account::new(
                0,
                "test".to_owned(),
                super::Network::Mainnet,
                derivation_path.clone(),
                AccountAddress::empty(super::Network::Mainnet),
                public_key,
            );

            let reconstructed_derivation_path = account.derivation_path();
            let derivation_index = account.derivation_index();

            assert_eq!(derivation_index, reconstructed_derivation_path[5]);
            assert_eq!(derivation_path, reconstructed_derivation_path);
        }
    }
}
