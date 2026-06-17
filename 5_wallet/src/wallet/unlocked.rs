use deps::{tokio::task::JoinHandle, *};
use secrets_store::SecretsStore;
use data_stores::{AppDataDb, DataBase};

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use bytes::Bytes;
use types::{
    Account, AppError, Notification, Persona, Resource, Transaction,
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
    /// The open app database, produced at the `Locked → Unlocked` transition. Owned here (not a
    /// process global), cloned into spawned tasks. See `.ai_docs/di_testability_plan.md` §1a.
    pub(crate) db: AppDataDb,
}

impl Unlocked {
    pub fn new(key: Key<DataBase>, db: AppDataDb) -> Self {
        Self { key, db }
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

    /// The injected capability bundle, for owned async tasks (signing/submit) that cannot borrow
    /// the wallet. See `.ai_docs/di_testability_plan.md`.
    pub fn env(&self) -> crate::env::Env {
        self.wallet_data.env.clone()
    }

    /// Convenience: the injected secrets store.
    pub fn secrets(&self) -> std::sync::Arc<dyn SecretsStore> {
        self.wallet_data.env.secrets.clone()
    }

    /// The open app database handle (for owned async read tasks that can't borrow the wallet).
    pub fn db(&self) -> AppDataDb {
        self.state.db.clone()
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

    /// Reads the persisted transaction history for an account (newest first by timestamp).
    /// Populated by the gateway sync (`upsert_transactions`); returns empty if none stored yet.
    pub async fn transactions_for_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<BTreeSet<Transaction>, AppError> {
        self.state
            .db
            .get_transactions_for_account::<BTreeSet<Transaction>>(account_address)
            .await
            .map_err(|err| AppError::NonFatal(Notification::Info(err.to_string())))
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
        let secrets = self.wallet_data.env.secrets.clone();
        let db = self.state.db.clone();

        deps::tokio::spawn(async move {
            let (mnemonic, seed_password) = crate::wallet::get_decrypted_mnemonic(&secrets, &password)?;
            let persona = crate::wallet::create_persona_from_mnemonic(
                &mnemonic,
                Some(seed_password.as_str()),
                id,
                index,
                label,
                network,
            );

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

    /// Updates a persona's user-controlled data (name, emails, phone numbers) in memory and
    /// persists it in the background. No mnemonic is needed — persona data is not derived secret
    /// material, only the identity address it is attached to is.
    pub fn update_persona_data(
        &mut self,
        identity_address: &str,
        persona_data: types::PersonaData,
    ) -> Result<(), AppError> {
        let persona = {
            let resource_data = Arc::make_mut(&mut self.wallet_data.resource_data);
            let persona = resource_data.personas.get_mut(identity_address).ok_or_else(|| {
                AppError::NonFatal(Notification::Info("Persona not found".to_string()))
            })?;
            persona.persona_data = persona_data;
            persona.clone()
        };
        let db = self.state.db.clone();
        deps::tokio::spawn(async move {
            let _ = db.upsert_persona(persona).await;
        });
        Ok(())
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

        let (mnemonic, seed_password) =
            crate::wallet::get_decrypted_mnemonic(&self.wallet_data.env.secrets, &password)?;
        let persona = crate::wallet::create_persona_from_mnemonic(
            &mnemonic,
            Some(seed_password.as_str()),
            id,
            index,
            label,
            network,
        );

        let db = self.state.db.clone();

        Arc::make_mut(&mut self.wallet_data.resource_data)
            .save_persona(persona.clone(), &db)
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

        // Injected per-network gateway provider + secrets (see .ai_docs/di_testability_plan.md §1a).
        let gateway = self.wallet_data.env.gateway(from_account.network);
        let secrets = self.wallet_data.env.secrets.clone();

        crate::transaction::service::submit_transfer_with_password(
            request,
            from_account,
            gateway,
            secrets,
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
        let salt = self.wallet_data.env.secrets.get_db_encryption_salt()?;
        let key = Key::new(password.as_str(), &salt);
        let db = self.state.db.clone();
        Ok(self
            .wallet_data
            .create_new_account(account_name, password, key, db))
    }
}
