use crate::statements;

use super::{statements::upsert, Db};
use anyhow::Result;
use rusqlite::params;
use types::{
    assets::FungibleAsset, Account, AccountAddress, EntityAccount, Fungibles, NonFungibles,
};

use super::AsyncDb;

impl Db {
    pub fn update_account(&mut self, account: &Account) -> Result<(), rusqlite::Error> {
        self.connection
            .prepare_cached(upsert::UPSERT_ACCOUNT)?
            .execute(params![
                account.address,
                account.id as i64,
                account.name,
                account.network,
                account.derivation_path,
                account.public_key.0,
                account.hidden,
                account.settings,
            ])?;

        Ok(())
    }

    pub fn update_accounts(&mut self, accounts: &[EntityAccount]) -> Result<(), rusqlite::Error> {
        let tx = self.connection.transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "
                UPDATE accounts SET name = ?, settings =?
                WHERE address = ?
                ",
            )?;

            for account in accounts {
                stmt.execute(params![account.name, account.settings, account.address,])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn update_fungibles_for_account(
        &mut self,
        fungibles: &Fungibles,
        account_address: &AccountAddress,
    ) -> Result<(), rusqlite::Error> {
        let tx = self.connection.transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "
                INSERT INTO 
                fungibles (
                    address, 
                    name, 
                    symbol, 
                    icon, 
                    amount,
                    current_supply,
                    description, 
                    last_updated, 
                    metadata, 
                    account_address
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT (address)
                DO UPDATE SET 
                    name = excluded.name, 
                    symbol = excluded.symbol, 
                    icon = excluded.icon, 
                    amount = excluded.amount,
                    current_supply = excluded.current_supply,
                    description = excluded.description, 
                    last_updated = excluded.last_updated, 
                    metadata = excluded.metadata, 
                    account_address = excluded.account_address
                ",
            )?;

            for fungible in fungibles {
                stmt.execute(params![
                    fungible.address,
                    fungible.name,
                    fungible.symbol,
                    fungible.icon,
                    fungible.amount,
                    fungible.total_supply,
                    fungible.description,
                    fungible.last_updated_at_state_version,
                    fungible.metadata,
                    account_address,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn update_non_fungibles_for_account(
        &mut self,
        non_fungibles: &NonFungibles,
        account_address: &AccountAddress,
    ) -> Result<(), rusqlite::Error> {
        let tx = self.connection.transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "
                INSERT INTO 
                non_fungibles (
                    address, 
                    name, 
                    symbol, 
                    icon, 
                    description, 
                    nfids, 
                    last_updated, 
                    metadata, 
                    account_address
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT (address)
                DO UPDATE SET
                    name = excluded.name,
                    symbol = excluded.symbol,
                    icon = excluded.icon,
                    description = excluded.description,
                    nfids = excluded.nfids,
                    last_updated = excluded.last_updated,
                    metadata = excluded.metadata,
                    account_address = excluded.account_address
                ",
            )?;

            for non_fungible in non_fungibles {
                stmt.execute(params![
                    non_fungible.address,
                    non_fungible.name,
                    non_fungible.symbol,
                    non_fungible.icon,
                    non_fungible.description,
                    non_fungible.nfids,
                    non_fungible.last_updated_at_state_version,
                    non_fungible.metadata,
                    account_address,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn upsert_fungible_assets(
        &mut self,
        account_address: &AccountAddress,
        fungibles: &[FungibleAsset],
    ) -> Result<(), rusqlite::Error> {
        let tx = self.connection.transaction()?;

        {
            let mut stmt = tx.prepare_cached(upsert::UPSERT_FUNGIBLE_ASSET)?;

            for fungible_asset in fungibles {
                stmt.execute(params![
                    fungible_asset.id,
                    fungible_asset.resource_address,
                    fungible_asset.amount,
                    fungible_asset.last_updated,
                    account_address,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }
}

impl AsyncDb {
    pub async fn update_account(&mut self, account: Account) -> Result<(), rusqlite::Error> {
        self.connection
            .call_unwrap(move |conn| {
                conn.prepare_cached(upsert::UPSERT_ACCOUNT)?
                    .execute(params![
                        account.address,
                        account.id as i64,
                        account.name,
                        account.network,
                        account.derivation_path,
                        account.public_key.0,
                        account.hidden,
                        account.settings,
                    ])?;
                Ok::<(), rusqlite::Error>(())
            })
            .await?;

        Ok(())
    }

    pub async fn update_accounts(
        &mut self,
        accounts: Vec<EntityAccount>,
    ) -> Result<(), rusqlite::Error> {
        self.connection
            .call_unwrap(move |conn| {
                let tx = conn.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(
                        "
                    UPDATE accounts SET name = ?, settings =?
                    WHERE address = ?
                    ",
                    )?;

                    for account in accounts {
                        stmt.execute(params![account.name, account.settings, account.address,])?;
                    }
                }

                tx.commit()?;
                Ok::<(), rusqlite::Error>(())
            })
            .await?;

        Ok(())
    }

    pub async fn update_fungibles_for_account(
        &mut self,
        fungibles: Fungibles,
        account_address: AccountAddress,
    ) -> Result<(), rusqlite::Error> {
        self.connection
            .call_unwrap(move |conn| {
                let tx = conn.transaction()?;
                {
                    let mut stmt = tx.prepare_cached(
                        "
                    INSERT INTO 
                    fungibles (
                        address, 
                        name, 
                        symbol, 
                        icon, 
                        amount,
                        current_supply,
                        description, 
                        last_updated, 
                        metadata, 
                        account_address
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    ON CONFLICT (address)
                    DO UPDATE SET 
                        name = excluded.name, 
                        symbol = excluded.symbol, 
                        icon = excluded.icon, 
                        amount = excluded.amount,
                        current_supply = excluded.current_supply,
                        description = excluded.description, 
                        last_updated = excluded.last_updated, 
                        metadata = excluded.metadata, 
                        account_address = excluded.account_address
                    ",
                    )?;

                    for fungible in &fungibles {
                        stmt.execute(params![
                            fungible.address,
                            fungible.name,
                            fungible.symbol,
                            fungible.icon,
                            fungible.amount,
                            fungible.total_supply,
                            fungible.description,
                            fungible.last_updated_at_state_version,
                            fungible.metadata,
                            &account_address,
                        ])?;
                    }
                }

                tx.commit()?;
                Ok::<(), rusqlite::Error>(())
            })
            .await?;

        Ok(())
    }

    pub async fn update_non_fungibles_for_account(
        &mut self,
        non_fungibles: NonFungibles,
        account_address: AccountAddress,
    ) -> Result<(), rusqlite::Error> {
        self.connection
            .call_unwrap(move |conn| {
                let tx = conn.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(
                        "
                    INSERT INTO 
                    non_fungibles (
                        address, 
                        name, 
                        symbol, 
                        icon, 
                        description, 
                        nfids, 
                        last_updated, 
                        metadata, 
                        account_address
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                    ON CONFLICT (address)
                    DO UPDATE SET
                        name = excluded.name,
                        symbol = excluded.symbol,
                        icon = excluded.icon,
                        description = excluded.description,
                        nfids = excluded.nfids,
                        last_updated = excluded.last_updated,
                        metadata = excluded.metadata,
                        account_address = excluded.account_address,
                    ",
                    )?;

                    for non_fungible in &non_fungibles {
                        stmt.execute(params![
                            non_fungible.address,
                            non_fungible.name,
                            non_fungible.symbol,
                            non_fungible.icon,
                            non_fungible.description,
                            non_fungible.nfids,
                            non_fungible.last_updated_at_state_version,
                            non_fungible.metadata,
                            account_address,
                        ])?;
                    }
                }

                tx.commit()?;
                Ok::<(), rusqlite::Error>(())
            })
            .await?;

        Ok(())
    }
}
