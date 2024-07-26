use super::{
    statements::{insert, upsert},
    AsyncDb, Db,
};
use rusqlite::params;
use types::{
    address::AccountAddress,
    assets::{FungibleAsset, NonFungibleAsset},
    crypto::HashedPassword,
    Account, Resource, Transaction,
};

impl Db {
    pub fn upsert_password_hash(&mut self, hash: HashedPassword) -> Result<(), rusqlite::Error> {
        self.connection
            .prepare_cached(upsert::UPSERT_PASSWORD_HASH)?
            .execute(params![1, hash])?;

        Ok(())
    }

    pub fn upsert_account(&mut self, account: &Account) -> Result<(), rusqlite::Error> {
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
                account.balances_last_updated,
                account.transactions_last_updated,
            ])?;

        Ok(())
    }

    pub fn upsert_resources(&mut self, resources: &[Resource]) -> Result<(), rusqlite::Error> {
        let tx = self.connection.transaction()?;

        {
            let mut stmt = tx.prepare_cached(upsert::UPSERT_RESOURCE)?;

            for resource in resources {
                stmt.execute(params![
                    resource.address,
                    resource.name,
                    resource.symbol,
                    resource.description,
                    resource.current_supply,
                    resource.divisibility,
                    resource.tags,
                ])?;
            }
        }

        tx.commit()
    }

    pub fn upsert_fungible_assets_for_account(
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
                    account_address,
                ])?;
            }
        }

        tx.commit()
    }

    pub fn upsert_non_fungible_assets_for_account(
        &mut self,
        account_address: &AccountAddress,
        non_fungibles: &[NonFungibleAsset],
    ) -> Result<(), rusqlite::Error> {
        let tx = self.connection.transaction()?;

        {
            let mut stmt = tx.prepare_cached(upsert::UPSERT_NON_FUNGIBLE_ASSET)?;

            for non_fungible_asset in non_fungibles {
                stmt.execute(params![
                    non_fungible_asset.id,
                    non_fungible_asset.resource_address,
                    non_fungible_asset.nfids,
                    account_address,
                ])?;
            }
        }

        tx.commit()
    }

    pub fn update_transaction_status(
        &mut self,
        transactions: &[Transaction],
    ) -> Result<(), rusqlite::Error> {
        let tx = self.connection.transaction()?;

        {
            let mut stmt = tx.prepare_cached(&upsert::UPSERT_TRANSACTION)?;

            for transaction in transactions {
                stmt.execute(params![
                    transaction.id,
                    transaction.transaction_address,
                    transaction.timestamp,
                    transaction.state_version as i64,
                ])?;
            }
        }

        tx.commit()
    }

    fn insert_transactions(&mut self, transactions: &[Transaction]) -> Result<(), rusqlite::Error> {
        let tx = self.connection.transaction()?;

        {
            let mut transaction_stmt = tx.prepare_cached(upsert::UPSERT_TRANSACTION)?;
            let mut balance_changes_stmt = tx.prepare_cached(insert::INSERT_BALANCE_CHANGE)?;

            for transaction in transactions {
                transaction_stmt.execute(params![
                    transaction.id,
                    transaction.transaction_address,
                    transaction.timestamp,
                    transaction.state_version as i64,
                ])?;

                for balance_change in &transaction.balance_changes {
                    balance_changes_stmt.execute(params![
                        balance_change.id,
                        balance_change.account,
                        balance_change.resource,
                        balance_change.nfids,
                        balance_change.amount,
                        transaction.transaction_address,
                    ])?;
                }
            }
        }

        tx.commit()
    }
}

impl AsyncDb {
    pub async fn upsert_password_hash(
        &self,
        hash: HashedPassword,
    ) -> Result<(), async_sqlite::Error> {
        self.client
            .conn(move |conn| {
                conn.prepare_cached(upsert::UPSERT_PASSWORD_HASH)?
                    .execute(params![1, hash])?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
    }

    pub async fn upsert_account(&self, account: Account) -> Result<(), async_sqlite::Error> {
        self.client
            .conn(move |conn| {
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
                        account.balances_last_updated,
                        account.transactions_last_updated,
                    ])?;
                Ok::<(), rusqlite::Error>(())
            })
            .await
    }

    pub async fn upsert_resources(
        &self,
        resources: Vec<Resource>,
    ) -> Result<(), async_sqlite::Error> {
        self.client
            .conn_mut(move |conn| {
                let tx = conn.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(upsert::UPSERT_RESOURCE)?;

                    for resource in resources.iter() {
                        stmt.execute(params![
                            resource.address,
                            resource.name,
                            resource.symbol,
                            resource.description,
                            resource.current_supply,
                            resource.divisibility,
                            resource.tags,
                        ])?;
                    }
                }

                tx.commit()
            })
            .await
    }

    pub async fn upsert_fungible_assets_for_account(
        &self,
        account_address: AccountAddress,
        fungibles: Vec<FungibleAsset>,
    ) -> Result<(), async_sqlite::Error> {
        self.client
            .conn_mut(move |connection| {
                let tx = connection.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(upsert::UPSERT_FUNGIBLE_ASSET)?;

                    for fungible_asset in fungibles {
                        stmt.execute(params![
                            fungible_asset.id,
                            fungible_asset.resource_address,
                            fungible_asset.amount,
                            account_address,
                        ])?;
                    }
                }

                tx.commit()
            })
            .await
    }

    pub async fn upsert_non_fungible_assets_for_account(
        &self,
        account_address: AccountAddress,
        non_fungibles: Vec<NonFungibleAsset>,
    ) -> Result<(), async_sqlite::Error> {
        self.client
            .conn_mut(move |connection| {
                let tx = connection.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(upsert::UPSERT_NON_FUNGIBLE_ASSET)?;

                    for non_fungible_asset in non_fungibles {
                        stmt.execute(params![
                            non_fungible_asset.id,
                            non_fungible_asset.resource_address,
                            non_fungible_asset.nfids,
                            account_address,
                        ])?;
                    }
                }

                tx.commit()
            })
            .await
    }

    pub async fn update_transaction_status(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<(), async_sqlite::Error> {
        self.client
            .conn_mut(move |conn| {
                let tx = conn.transaction()?;

                {
                    let mut stmt = tx.prepare_cached(&upsert::UPSERT_TRANSACTION)?;

                    for transaction in transactions {
                        stmt.execute(params![
                            transaction.id,
                            transaction.transaction_address,
                            transaction.timestamp,
                            transaction.state_version as i64,
                        ])?;
                    }
                }

                tx.commit()
            })
            .await
    }

    pub async fn insert_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<(), async_sqlite::Error> {
        self.client
            .conn_mut(|conn| {
                let tx = conn.transaction()?;

                {
                    let mut transaction_stmt = tx.prepare_cached(upsert::UPSERT_TRANSACTION)?;
                    let mut balance_changes_stmt =
                        tx.prepare_cached(insert::INSERT_BALANCE_CHANGE)?;

                    for transaction in transactions {
                        transaction_stmt.execute(params![
                            transaction.id,
                            transaction.transaction_address,
                            transaction.timestamp,
                            transaction.state_version as i64,
                        ])?;

                        for balance_change in &transaction.balance_changes {
                            balance_changes_stmt.execute(params![
                                balance_change.id,
                                balance_change.account,
                                balance_change.resource,
                                balance_change.nfids,
                                balance_change.amount,
                                transaction.transaction_address,
                            ])?;
                        }
                    }
                }

                tx.commit()
            })
            .await
    }
}
