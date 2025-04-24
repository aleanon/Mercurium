use deps::*;
use store::{AppDataDb, DbError, IconsDb};

use std::collections::{BTreeSet, HashMap};

use bytes::Bytes;
use types::{address::{AccountAddress, ResourceAddress}, assets::{FungibleAsset, NonFungibleAsset}, debug_info, Account, Resource, UnwrapUnreachable};

#[derive(Debug, Clone)]
pub struct ResourceData {
    pub accounts: HashMap<AccountAddress, Account>,
    pub fungibles: HashMap<AccountAddress, BTreeSet<FungibleAsset>>,
    pub non_fungibles: HashMap<AccountAddress, BTreeSet<NonFungibleAsset>>,
    pub resources: HashMap<ResourceAddress, Resource>,
    pub resource_icons: HashMap<ResourceAddress, Bytes>,
}


impl ResourceData {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            fungibles: HashMap::new(),
            non_fungibles: HashMap::new(),
            resources: HashMap::new(),
            resource_icons: HashMap::new(),
        }
    }

    pub async fn load_resource_data_from_disk(&mut self, app_data_db: &AppDataDb, icons_db: &IconsDb) -> Result<(), DbError> {
        self.accounts = app_data_db.get_accounts().await?;
        self.fungibles = app_data_db.get_all_fungible_assets_per_account().await?;
        self.non_fungibles = app_data_db.get_all_non_fungible_assets_per_account().await?;
        self.resources = app_data_db.get_all_resources().await?;
        self.resource_icons = handles::store::get::resource_icons(&icons_db).await;

        Ok(())
    }

    pub async fn save_resource_data_to_disk(&self, db: &AppDataDb) -> Result<(), DbError> {
        self.save_accounts_to_disk(db).await
            .inspect_err(|err| eprintln!("Failed to save accounts: {err}"))?;
        self.save_resources_to_disk(db).await
            .inspect_err(|err| eprintln!("Failed to save resources: {err}"))?;
        self.save_fungibles_to_disk(db).await
            .inspect_err(|err| eprintln!("Failed to save fungibles: {err}"))?;
        self.save_non_fungibles_to_disk(db).await
            .inspect_err(|err| eprintln!("Failed to save non fungibles: {err}"))?;
        Ok(())
    }

    pub async fn save_accounts_to_disk(&self, db: &AppDataDb) -> Result<(), DbError> {
        let accounts = self.accounts.values().cloned().collect::<Vec<_>>();
        db.upsert_accounts(accounts).await?;
        Ok(())
    }

    pub async fn save_fungibles_to_disk(&self, db: &AppDataDb) -> Result<(), DbError> {
        for (account_address, fungibles) in &self.fungibles {
            db.upsert_fungible_assets_for_account(account_address.clone(), fungibles.clone()).await?;
        }
        Ok(())
    }

    pub async fn save_non_fungibles_to_disk(&self, db: &AppDataDb) -> Result<(), DbError> {
        for (account_address, non_fungibles) in &self.non_fungibles {
            db.upsert_non_fungible_assets_for_account(account_address.clone(), non_fungibles.clone()).await?;
        }
        Ok(())
    }

    pub async fn save_resources_to_disk(&self, db: &AppDataDb) -> Result<(), DbError> {
        let resources = self.resources.values().cloned().collect::<Vec<_>>();
        db.upsert_resources(resources).await?;
        Ok(())
    }

    pub async fn set_resource_icons(&mut self, icons: HashMap<ResourceAddress, Vec<u8>>) {
        let icons = icons.into_iter().map(|(address, data)| {
            (address.clone(), Bytes::from_owner(data))
        })
        .collect();

        self.resource_icons = icons
    }

    pub async fn save_account(&mut self, account: Account, db: &AppDataDb) -> Result<(), DbError> {
        self.accounts.insert(account.address.clone(), account.clone()).unwrap_unreachable(debug_info!("Created an account that already exists"));
        db.upsert_account(account).await
    }

}