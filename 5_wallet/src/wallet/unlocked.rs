use deps::{tokio::task::JoinHandle, *};
use secrets_store::get_db_encryption_salt;
use store::{AppDataDb, DataBase};

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use bytes::Bytes;
use types::{
    Account, AppError, Notification, Persona, Resource,
    address::{AccountAddress, ResourceAddress},
    assets::{FungibleAsset, NonFungibleAsset},
    crypto::{Key, Password},
};

use crate::{
    Settings,
    transaction::{TransferRequest, service::SubmittedTransaction},
    wallet::WalletState,
};

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

    pub fn personas(&self) -> &HashMap<String, Persona> {
        &self.wallet_data.resource_data.personas
    }

    /// Spawns derivation + persistence of a new persona, returning a handle to await (mirrors
    /// [`Self::create_new_account`]). The persona is **not** inserted into the in-memory map here;
    /// the caller registers it with [`Self::register_persona`] once the handle resolves. This
    /// shape fits an iced `Task` (no borrow of the wallet is captured by the future).
    pub fn create_persona_handle(
        &self,
        label: String,
        password: Password,
    ) -> JoinHandle<Result<Persona, AppError>> {
        let (id, index) = self.wallet_data.resource_data.personas.values().fold(
            (0i64, 0u32),
            |(mut id, mut index), persona| {
                if persona.id >= id {
                    id = persona.id + 1;
                }
                let der_index = persona.derivation_index();
                if der_index >= index {
                    index = der_index + 1;
                }
                (id, index)
            },
        );
        let network = self.wallet_data.settings.network;

        deps::tokio::spawn(async move {
            let (mnemonic, seed_password) = crate::wallet::get_decrypted_mnemonic(&password)?;
            let persona = crate::wallet::create_persona_from_mnemonic(
                &mnemonic,
                Some(seed_password.as_str()),
                id,
                index,
                label,
                network,
            );

            let db = AppDataDb::get(network)
                .ok_or_else(|| AppError::Fatal("Database not found".to_string()))?;
            db.upsert_persona(persona.clone())
                .await
                .map_err(|err| AppError::NonFatal(Notification::Info(err.to_string())))?;
            Ok(persona)
        })
    }

    /// Inserts an already-created persona into the in-memory map (call after
    /// [`Self::create_persona_handle`] resolves successfully).
    pub fn register_persona(&mut self, persona: Persona) {
        Arc::make_mut(&mut self.wallet_data.resource_data)
            .personas
            .insert(persona.identity_address.clone(), persona);
    }

    /// Derives, persists and registers a new persona (identity). The mnemonic is re-decrypted
    /// for this operation using the login `password`, then dropped.
    pub async fn create_new_persona(
        &mut self,
        label: String,
        password: Password,
    ) -> Result<Persona, AppError> {
        let network = self.wallet_data.settings.network;

        // Next free id / derivation index, mirroring account creation.
        let (id, index) = self.wallet_data.resource_data.personas.values().fold(
            (0i64, 0u32),
            |(mut id, mut index), persona| {
                if persona.id >= id {
                    id = persona.id + 1;
                }
                let der_index = persona.derivation_index();
                if der_index >= index {
                    index = der_index + 1;
                }
                (id, index)
            },
        );

        let (mnemonic, seed_password) = crate::wallet::get_decrypted_mnemonic(&password)?;
        let persona = crate::wallet::create_persona_from_mnemonic(
            &mnemonic,
            Some(seed_password.as_str()),
            id,
            index,
            label,
            network,
        );

        let db = AppDataDb::get(network)
            .ok_or_else(|| AppError::Fatal("Database not found".to_string()))?;

        Arc::make_mut(&mut self.wallet_data.resource_data)
            .save_persona(persona.clone(), db)
            .await
            .map_err(|err| AppError::NonFatal(Notification::Info(err.to_string())))?;

        Ok(persona)
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

    /// Builds, signs, notarizes and submits a transfer from one of this wallet's accounts.
    ///
    /// The mnemonic is re-decrypted from the OS credential store for this signing operation
    /// (using the login `password`) and dropped immediately afterwards — it is not cached in the
    /// unlocked wallet. Returns the submitted transaction id for status polling; does not wait
    /// for commitment (use [`crate::transaction::service::poll_until_settled`]).
    pub async fn send_transfer(
        &self,
        request: TransferRequest,
        password: Password,
        tip_percentage: u16,
    ) -> Result<SubmittedTransaction, AppError> {
        let from_account = self
            .accounts()
            .get(&request.from)
            .ok_or_else(|| {
                AppError::NonFatal(Notification::Info(
                    "Sending account is not in this wallet".to_string(),
                ))
            })?
            .clone();

        crate::transaction::service::submit_transfer_with_password(
            request,
            from_account,
            password,
            tip_percentage,
        )
        .await
    }

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
