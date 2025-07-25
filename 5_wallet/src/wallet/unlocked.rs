use deps::{tokio::task::JoinHandle, *};
use handles::credentials::get_db_encryption_salt;
use store::DataBase;

use std::collections::{BTreeSet, HashMap};

use bytes::Bytes;
use types::{
    Account, AppError, Resource,
    address::{AccountAddress, ResourceAddress},
    assets::{FungibleAsset, NonFungibleAsset},
    crypto::{Key, Password},
};

use crate::{Settings, wallet::WalletState};

use super::{Wallet, locked::Locked};

#[derive(Clone)]
pub struct Unlocked {
    pub(crate) key: Key<DataBase>,
}

impl Unlocked {
    pub fn new(key: Key<DataBase>) -> Self {
        Self { key }
    }
}

impl WalletState for Unlocked {}

impl Wallet<Unlocked> {
    pub fn logout(self) -> Wallet<Locked> {
        Wallet {
            state: Locked::new(false),
            wallet_data: self.wallet_data,
        }
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.wallet_data.settings
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

    // pub fn accounts_mut(&mut self) -> &mut HashMap<AccountAddress, Account> {
    //     &mut self.wallet_data.resource_data.accounts
    // }

    pub fn create_new_account(
        &mut self,
        account_name: String,
        password: Password,
    ) -> Result<JoinHandle<Result<Account, AppError>>, AppError> {
        let salt = get_db_encryption_salt()?;
        let key = Key::new(password.as_str(), &salt);
        Ok(self
            .wallet_data
            .create_new_account(account_name, password, key))
    }
}
