use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use debug_print::debug_eprintln;
use store::DataBase;
use tokio::sync::Mutex;
use types::{address::ResourceAddress, collections::AccountsUpdate, crypto::{bip39::Mnemonic, EncryptedMnemonic, KeySaltPair, Password}, Account, AccountSummary, Network, TaskResponse};

use crate::wallet_keys_and_salt::WalletEncryptionKeys;



pub struct SetupTasks {
    pub mnemonic_key_salt: Arc<Mutex<TaskResponse<KeySaltPair<EncryptedMnemonic>>>>,
    pub db_key_and_salt: Arc<Mutex<TaskResponse<KeySaltPair<DataBase>>>>,
    pub accounts: Arc<Mutex<TaskResponse<Vec<(Account, AccountSummary)>>>>,
    pub accounts_update: Arc<Mutex<TaskResponse<AccountsUpdate>>>,
    pub icons_data: Arc<Mutex<TaskResponse<HashMap<ResourceAddress, Bytes>>>>,
}


impl SetupTasks {
    pub fn new() -> Self {
        Self {
            mnemonic_key_salt: Arc::new(Mutex::new(TaskResponse::new(0, None))),
            db_key_and_salt: Arc::new(Mutex::new(TaskResponse::new(0, None))),
            accounts: Arc::new(Mutex::new(TaskResponse::new(0, None))),
            accounts_update: Arc::new(Mutex::new(TaskResponse::new(0, None))),
            icons_data: Arc::new(Mutex::new(TaskResponse::new(0, None))),
        }
    }

    pub fn create_encryption_keys(&mut self, task_id: u8, password: Password) {
        let db_key_response = self.db_key_and_salt.clone();
        let mnemonic_key_response = self.mnemonic_key_salt.clone();
        

        tokio::spawn(async move {
            let mut db_key_response_lock = db_key_response.lock().await;
            let mut mnemonic_key_response_lock = mnemonic_key_response.lock().await;
            for _ in 0..3 {
                match WalletEncryptionKeys::new(&password) {
                    Ok(wallet_keys) => {
                        db_key_response_lock.new_response(TaskResponse::new(task_id, Some(wallet_keys.db_key_salt)));
                        mnemonic_key_response_lock.new_response(TaskResponse::new(task_id, Some(wallet_keys.mnemonic_key_salt)));
                        break;
                    }
                    Err(err) => {
                        debug_eprintln!("Unable to derive encryption keys: {err}");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });
    }

    pub fn create_and_update_accounts(&mut self, task_id: u8, mnemonic: Mnemonic, seed_password: Option<Password>, network: Network) {
        let accounts = self.accounts.clone();
        let accounts_update = self.accounts_update.clone();

        tokio::spawn(async move {
            let mut accounts_lock = accounts.lock().await;
            let mut accounts_update_lock = accounts_update.lock().await;

            let accounts_response = Self::create_accounts(task_id, mnemonic, seed_password, network).await;
            let Some(accounts_with_summary) = accounts_response.ref_data() else {
                return
            };

            let accounts_to_update = accounts_with_summary.iter()
                .map(|(account, _)| account.clone())
                .collect();
            
            let update_accounts_response = Self::update_accounts(task_id, accounts_to_update, network).await;

            accounts_lock.new_response(accounts_response);
            accounts_update_lock.new_response(update_accounts_response);
        });
    }

    pub async fn create_accounts(
        task_id: u8, 
        mnemonic: Mnemonic, 
        seed_password: Option<Password>, 
        network: Network
    ) -> TaskResponse<Vec<(Account, AccountSummary)>> {
        let password_as_str = seed_password
                .as_ref()
                .and_then(|password| Some(password.as_str()));

        let accounts = handles::wallet::create_multiple_accounts_from_mnemonic::<Vec<_>>(
            &mnemonic,
            password_as_str,
            0,
            0,
            60,
            network,
        );

        let accounts = accounts
            .into_iter()
            .map(|account| (account, AccountSummary::NoUpdateReceived))
            .collect::<Vec<(Account, AccountSummary)>>();

        TaskResponse::new(task_id, Some(accounts))
    }

    pub async fn update_accounts(task_id: u8, accounts: Vec<Account>, network: Network) -> TaskResponse<AccountsUpdate> {
        let accounts_update = handles::radix_dlt::updates::update_accounts(
            network,
            Arc::new(HashMap::new()),
            accounts,
        )
        .await;

        TaskResponse::new(task_id, Some(accounts_update))        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_update_accounts() {
        let mut setup_tasks = SetupTasks::new();
    }
}