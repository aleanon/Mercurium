use deps::{debug_print::debug_println, *};

use std::{collections::{BTreeMap, HashMap}, fmt::Debug, sync::Arc};

use debug_print::debug_eprintln;
use types::{address::ResourceAddress, collections::AccountsUpdate, crypto::{bip39::Mnemonic, Password}, Account, AccountSummary, Network};

use crate::{wallet_encryption_keys::WalletEncryptionKeys, SetupError};

use super::task_runner::TaskRunner;

pub struct TaskManager {
    pub wallet_keys_and_salt: TaskRunner<WalletEncryptionKeys, SetupError>,
    pub accounts: TaskRunner<Vec<(Account, AccountSummary)>, SetupError>,
    pub accounts_update: TaskRunner<AccountsUpdate, SetupError>,
    pub icons_data: TaskRunner<HashMap<ResourceAddress, (Vec<u8>, Vec<u8>)>, SetupError>,
}

impl Debug for TaskManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskManager")
    }
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            wallet_keys_and_salt: TaskRunner::new(),
            accounts: TaskRunner::new(),
            accounts_update: TaskRunner::new(),
            icons_data: TaskRunner::new(),
        }
    }

    pub async fn get_accounts_update(&self) -> Result<AccountsUpdate, SetupError> {
        Ok(self.accounts_update.get_result().await?)
    }

    pub async fn get_wallet_encryption_keys(&self) -> Result<WalletEncryptionKeys, SetupError> {
        Ok(self.wallet_keys_and_salt.get_result().await?)
    }

    pub async fn get_accounts_with_summary(&self) -> Result<Vec<(Account, AccountSummary)>, SetupError> {
        Ok(self.accounts.get_result().await?)
    }

    pub async fn get_icons_data(&self) -> Result<HashMap<ResourceAddress, (Vec<u8>, Vec<u8>)>, SetupError> {
        Ok(self.icons_data.get_result().await?)
    }

    pub async fn run_task_create_encryption_keys(&self, task_id: u16, password: Password) {
        self.wallet_keys_and_salt.run_task(task_id,  move||Self::create_encryption_keys(password)).await;
    }

    pub async fn run_task_create_and_update_accounts(&self, task_id: u16, mnemonic: Mnemonic, seed_password: Option<Password>, network: Network) {
        let accounts = Self::create_accounts(mnemonic, seed_password, network).await;

        self.accounts_update.run_task(task_id, move || Self::update_accounts(accounts, network)).await;
        let accounts_update = self.accounts_update.get_result().await.unwrap_or(AccountsUpdate::new(network));
        let icon_urls = accounts_update.icon_urls.clone();

        self.accounts.run_task(task_id,move || Self::accounts_with_summaries(accounts_update)).await;
        self.icons_data.run_task(task_id, move || Self::download_resource_icons(icon_urls)).await;
    }

    async fn create_encryption_keys(password: Password) -> Result<WalletEncryptionKeys, SetupError> {
        let mut error = SetupError::MissingDerivedKeys;
        for e in 0u8..=2 {
            match WalletEncryptionKeys::new(&password) {
                Ok(wallet_keys) => {
                    return Ok(wallet_keys);
                }
                Err(err) => {
                    debug_eprintln!("Unable to derive encryption keys: {err}");
                    error = err;
                    tokio::time::sleep(std::time::Duration::from_millis(10u64.pow(e as u32))).await;
                }
            }
        }
        Err(error)
    }

    async fn create_accounts(
        mnemonic: Mnemonic, 
        seed_password: Option<Password>, 
        network: Network
    ) -> Vec<Account> {
        let password_as_str = seed_password
                .as_ref()
                .and_then(|password| Some(password.as_str()));

        handles::wallet::create_multiple_accounts_from_mnemonic::<Vec<_>>(
            &mnemonic,
            password_as_str,
            0,
            0,
            60,
            network,
        )
    }

    async fn update_accounts(accounts: Vec<Account>, network: Network) -> Result<AccountsUpdate, SetupError> {
        Ok(handles::radix_dlt::updates::update_accounts(
            network,
            Arc::new(HashMap::new()),
            accounts,
        )
        .await)
    }

    async fn accounts_with_summaries(accounts_update: AccountsUpdate) -> Result<Vec<(Account, AccountSummary)>, SetupError> {
        let accounts = accounts_update.account_updates.iter()
            .map(|account_update| {

                let summary = if account_update.fungibles.len() == 0 && account_update.non_fungibles.len() == 0 {
                    AccountSummary::NoLedgerPresense
                } else {
                    AccountSummary::Summary {
                        nr_of_fungibles: account_update.fungibles.len(),
                        nr_of_non_fungibles: account_update.non_fungibles.len(),
                    }
                };

                (account_update.account.clone(), summary)
            })
            .collect();
        Ok(accounts)
    }

    async fn download_resource_icons(icon_urls: BTreeMap<ResourceAddress, String>) -> Result<HashMap<ResourceAddress, (Vec<u8>, Vec<u8>)>, SetupError> {
        debug_println!("Downloading icons");
        let result = handles::image::download::download_and_resize_icons(icon_urls).await;
        debug_println!("Icons downloaded");
        Ok(result)
    }
}