use std::{collections::HashMap, sync::Arc};

use deps::tokio::{self, task::JoinHandle};
use store::{AppDataDb, DataBase, DbError, IconsDb};
use types::{
    Account, AppError, Network,
    address::ResourceAddress,
    crypto::{EncryptedMnemonicError, Key, Password},
};

use crate::settings::Settings;

use super::{create_account_from_mnemonic, resource_data::ResourceData};

#[derive(Debug, Clone)]
pub struct WalletData {
    pub resource_data: Arc<ResourceData>,
    pub settings: Settings,
}

impl WalletData {
    pub fn new(settings: Settings) -> Self {
        Self {
            resource_data: Arc::new(ResourceData::new()),
            settings,
        }
    }

    pub fn set_network(&mut self, network: Network) {
        self.settings.network = network;
    }

    pub async fn save_resource_icons_to_disk(
        &self,
        icons: HashMap<ResourceAddress, Vec<u8>>,
        db_key: Key<DataBase>,
    ) -> Result<(), DbError> {
        let db = IconsDb::get_or_init(self.settings.network, db_key).await?;
        db.upsert_resource_icons(icons).await?;
        Ok(())
    }

    pub async fn save_resource_data_to_disk(&self, key: Key<DataBase>) -> Result<(), DbError> {
        let db = AppDataDb::get_or_init(self.settings.network, key).await?;

        self.resource_data.save_resource_data_to_disk(db).await
    }

    pub fn create_new_account(
        &mut self,
        account_name: String,
        password: Password,
        key: Key<DataBase>,
    ) -> JoinHandle<Result<Account, AppError>> {
        let (id, derivation_index) =
            self.resource_data
                .accounts
                .values()
                .fold((0, 0), |(mut id, mut index), account| {
                    if account.id >= id {
                        id = account.id + 1
                    }
                    let der_index = account.derivation_index();
                    if der_index >= index {
                        index = der_index + 1
                    };
                    (id, index)
                });
        let network = self.settings.network;

        tokio::spawn(async move {
            let encrypted_mnemonic = handles::credentials::get_encrypted_mnemonic()?;
            let (mnemonic, seed_password) = encrypted_mnemonic
                .decrypt_mnemonic(&password)
                .map_err(|err| match err {
                    EncryptedMnemonicError::FailedToDecryptData => {
                        AppError::NonFatal(types::Notification::Info("Wrong password".to_string()))
                    }
                    _ => AppError::Fatal(err.to_string()),
                })?;

            let account = create_account_from_mnemonic(
                &mnemonic,
                Some(seed_password.as_str()),
                id,
                derivation_index,
                account_name,
                network,
            );

            let db =
                AppDataDb::get(network).ok_or(AppError::Fatal("Database not found".to_string()))?;
            db.upsert_account(account.clone())
                .await
                .map_err(|err| AppError::NonFatal(types::Notification::Info(err.to_string())))?;
            Ok(account)
        })
    }
}
