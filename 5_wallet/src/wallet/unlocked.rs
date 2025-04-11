use deps_two::*;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use bytes::Bytes;
use types::{address::{AccountAddress, ResourceAddress}, assets::{FungibleAsset, NonFungibleAsset}, Account, Resource};

use crate::wallet::WalletState;

use super::{locked::Locked, wallet_data::WalletData, Wallet};

#[derive(Clone)]
pub struct Unlocked;



impl WalletState for Unlocked{}

impl Wallet<Unlocked> {
    pub fn logout(self) -> Wallet<Locked> {
        Wallet {state: Locked::new(false), wallet_data: self.wallet_data}
    }

    pub fn wallet_data_mut(&mut self) -> &mut WalletData {
        &mut self.wallet_data
    } 

    pub fn wallet_data(&self) -> &WalletData {
        &self.wallet_data
    }

    pub fn resources(&self) -> &HashMap<ResourceAddress, Resource> {
        &self.wallet_data.resource_data.resources
    }

    pub fn accounts(&self) -> &HashMap<AccountAddress, Account> {
        &self.wallet_data.resource_data.accounts
    }

    pub fn fungibles(&self) -> &HashMap<AccountAddress, BTreeSet<FungibleAsset>> {
        &self.wallet_data.resource_data.fungibles
    }

    pub fn non_fungibles(&self) -> &HashMap<AccountAddress, BTreeSet<NonFungibleAsset>> {
        &self.wallet_data.resource_data.non_fungibles
    }

    pub fn resource_icons(&self) -> &HashMap<ResourceAddress, Bytes> {
        &self.wallet_data.resource_data.resource_icons
    }

    pub fn accounts_mut(&mut self) -> &mut HashMap<AccountAddress, Account> {
        &mut self.wallet_data.resource_data.accounts
    }
}