use std::collections::BTreeSet;

use serde::{Serialize, Deserialize};

use crate::{AccountAddress, ResourceAddress};




#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub(in super) third_party_deposits: DepositRules,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            third_party_deposits: DepositRules::default()
        }
    }
}

impl rusqlite::types::FromSql for Settings {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(blob) => {
                Ok(
                    serde_json::from_slice(blob)
                        .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?
                )
            }
            _ => Err(rusqlite::types::FromSqlError::InvalidType)
        }
    }
}


impl rusqlite::types::ToSql for Settings {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(
            rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Blob(serde_json::to_vec(self)
                .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?))
        )
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
    pub fn new () -> Self {
        Self::default()
    }

    pub fn accept_deposits(&mut self, rule: AcceptDeposits) {
        self.accept_deposits = rule
    }

    pub fn add_asset_allow(&mut self, resource_address: ResourceAddress) {
        match self.allow_specific {
            Some(ref mut tree) => tree.insert(resource_address),
            None => self.allow_specific.insert(BTreeSet::new()).insert(resource_address),
        };
    }

    pub fn add_asset_deny(&mut self, resource_address: ResourceAddress) {
        match self.deny_specific {
            Some(ref mut tree) => tree.insert(resource_address),
            None => self.deny_specific.insert(BTreeSet::new()).insert(resource_address),
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
            None => self.allow_depositors.insert(BTreeSet::new()).insert(depositor_address),
        };
    }

    pub fn remove_depositor(&mut self, depositor_address: AccountAddress) {
        if let Some(ref mut tree) = self.allow_depositors {
            tree.remove(&depositor_address);
            if tree.len() == 0 {self.allow_depositors = None};
        }
    }

    pub fn show_rules(&self) -> (&AcceptDeposits, &Option<BTreeSet<ResourceAddress>>, &Option<BTreeSet<ResourceAddress>>, &Option<BTreeSet<AccountAddress>>) {
        (&self.accept_deposits, &self.allow_specific, &self.deny_specific, &self.allow_depositors)
    }
}

impl Default for DepositRules {
    fn default() -> Self {
        Self { accept_deposits: AcceptDeposits::All, allow_specific: None, deny_specific: None, allow_depositors: None}
    }
}

#[derive(Debug,Clone,Default, Serialize,Deserialize, PartialEq, Eq)]
pub enum AcceptDeposits {
    #[default]
    All,
    Known,
    DenyAll
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
