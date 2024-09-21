use super::Db;
use async_sqlite::rusqlite;
use asynciter::{AsyncIterator, FromAsyncIterator, IntoAsyncIterator};
use std::collections::{BTreeSet, HashMap};
use types::{
    address::AccountAddress,
    assets::{FungibleAsset, NonFungibleAsset},
    crypto::HashedPassword,
    Account, BalanceChange, Ed25519PublicKey, Resource, Transaction, TransactionId,
};

impl Db {
    pub async fn get_db_password_hash(&self) -> Result<HashedPassword, async_sqlite::Error> {
        self.client
            .conn(|conn| {
                conn.prepare_cached("SELECT password FROM password_hash WHERE id = 1")?
                    .query_row([], |row| Ok(row.get(0)?))
            })
            .await
    }

    pub async fn get_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<Account, async_sqlite::Error> {
        self.client
            .conn(|conn| {
                conn.prepare_cached("SELECT * FROM accounts WHERE address = ?")?
                    .query_row([account_address], |row| {
                        Ok(Account {
                            address: row.get(0)?,
                            id: row.get(1)?,
                            name: row.get(2)?,
                            network: row.get(3)?,
                            derivation_path: row.get(4)?,
                            public_key: Ed25519PublicKey(row.get(5)?),
                            hidden: row.get(6)?,
                            settings: row.get(7)?,
                            balances_last_updated: row.get(8)?,
                            transactions_last_updated: row.get(9)?,
                        })
                    })
            })
            .await
    }

    pub async fn get_account_addresses<T>(&self) -> Result<T, async_sqlite::Error>
    where
        T: FromIterator<AccountAddress> + Send + 'static,
    {
        self.client
            .conn(|conn| {
                conn.prepare_cached("SELECT address FROM accounts")?
                    .query_map([], |row| Ok(row.get(0)?))?
                    .collect()
            })
            .await
    }

    pub async fn get_accounts<T>(&self) -> Result<T, async_sqlite::Error>
    where
        T: FromIterator<Account> + Send + 'static,
    {
        self.client
            .conn(|conn| {
                conn.prepare_cached("SELECT * FROM accounts")?
                    .query_map([], |row| {
                        let account = Account {
                            address: row.get(0)?,
                            id: row.get(1)?,
                            name: row.get(2)?,
                            network: row.get(3)?,
                            derivation_path: row.get(4)?,
                            public_key: Ed25519PublicKey(row.get(5)?),
                            hidden: row.get(6)?,
                            settings: row.get(7)?,
                            balances_last_updated: row.get(8)?,
                            transactions_last_updated: row.get(9)?,
                        };
                        Ok(account)
                    })?
                    .collect()
            })
            .await
    }

    pub async fn get_all_fungible_assets_per_account<T, U>(&self) -> Result<T, async_sqlite::Error>
    where
        T: FromIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<FungibleAsset> + Send + 'static,
    {
        let accounts = self.get_account_addresses::<Vec<_>>().await?;

        self.client
            .conn(|conn| {
                let mut stmt =
                    conn.prepare_cached("SELECT * FROM fungible_assets WHERE account_address = ?")?;

                accounts
                    .into_iter()
                    .map(|account_address| {
                        stmt.query_map([&account_address], |row| {
                            let fungible_asset = FungibleAsset {
                                id: row.get(0)?,
                                resource_address: row.get(1)?,
                                amount: row.get(2)?,
                            };

                            Ok(fungible_asset)
                        })?
                        .collect::<Result<_, _>>()
                        .map(|assets| (account_address, assets))
                    })
                    .collect()
            })
            .await
    }

    pub async fn get_fungible_assets_for_accounts<T, U>(
        &self,
        account_addresses: Vec<AccountAddress>,
    ) -> Result<T, async_sqlite::Error>
    where
        T: FromIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<FungibleAsset> + Send + 'static,
    {
        self.client
            .conn(|conn| {
                let mut stmt =
                    conn.prepare_cached("SELECT * FROM fungible_assets WHERE account_address = ?")?;

                account_addresses
                    .into_iter()
                    .map(|account_address| {
                        let fungible_assets = stmt
                            .query_map([&account_address], |row| {
                                let non_fungible_asset = FungibleAsset {
                                    id: row.get(0)?,
                                    resource_address: row.get(1)?,
                                    amount: row.get(2)?,
                                };

                                Ok(non_fungible_asset)
                            })?
                            .collect::<Result<U, rusqlite::Error>>()?;

                        Ok((account_address, fungible_assets))
                    })
                    .collect()
            })
            .await
    }

    pub async fn get_all_non_fungible_assets_per_account<T, U>(
        &self,
    ) -> Result<T, async_sqlite::Error>
    where
        T: FromIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        let accounts = self.get_account_addresses::<Vec<_>>().await?;

        self.client
            .conn(|conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT * FROM non_fungible_assets WHERE account_address = ?",
                )?;

                accounts
                    .into_iter()
                    .map(|account_address| {
                        let non_fungible_assets = stmt
                            .query_map([&account_address], |row| {
                                let non_fungible_asset = NonFungibleAsset {
                                    id: row.get(0)?,
                                    resource_address: row.get(1)?,
                                    nfids: row.get(2)?,
                                };

                                Ok(non_fungible_asset)
                            })?
                            .collect::<Result<U, rusqlite::Error>>()?;

                        Ok((account_address, non_fungible_assets))
                    })
                    .collect()
            })
            .await
    }

    pub async fn get_non_fungible_assets_for_accounts<T, U>(
        &self,
        account_addresses: Vec<AccountAddress>,
    ) -> Result<T, async_sqlite::Error>
    where
        T: FromIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        self.client
            .conn(|conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT * FROM non_fungible_assets WHERE account_address = ?",
                )?;

                account_addresses
                    .into_iter()
                    .map(|account_address| {
                        let non_fungible_assets = stmt
                            .query_map([&account_address], |row| {
                                let non_fungible_asset = NonFungibleAsset {
                                    id: row.get(0)?,
                                    resource_address: row.get(1)?,
                                    nfids: row.get(2)?,
                                };

                                Ok(non_fungible_asset)
                            })?
                            // .filter_map(|result| result.ok())
                            .collect::<Result<U, rusqlite::Error>>()?;

                        Ok((account_address, non_fungible_assets))
                    })
                    .collect()
            })
            .await
    }

    pub async fn get_all_resources<T>(&self) -> Result<T, async_sqlite::Error>
    where
        T: FromIterator<Resource> + Send + 'static,
    {
        self.client
            .conn(|conn| {
                conn.prepare_cached("SELECT * FROM resources")?
                    .query_map([], |row| {
                        let resource = Resource {
                            address: row.get(0)?,
                            name: row.get(1)?,
                            symbol: row.get(2)?,
                            description: row.get(3)?,
                            current_supply: row.get(4)?,
                            divisibility: row.get(5)?,
                            tags: row.get(6)?,
                        };
                        Ok(resource)
                    })?
                    .collect()
            })
            .await
    }

    pub async fn get_last_transaction_for_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<Transaction, async_sqlite::Error> {
        let mut transactions: BTreeSet<Transaction> =
            self.get_transactions_for_account(account_address).await?;
        transactions
            .pop_last()
            .ok_or(rusqlite::Error::QueryReturnedNoRows.into())
    }

    pub async fn get_transactions_for_account<T>(
        &self,
        account_address: AccountAddress,
    ) -> Result<T, async_sqlite::Error>
    where
        T: FromAsyncIterator<Transaction> + Send + 'static,
    {
        let balance_changes = self
            .get_balance_changes_for_account(account_address)
            .await?;

        let transactions: T = balance_changes
            .into_aiter()
            .afilter_map(|(transaction_id, balance_changes)| async move {
                self.client
                    .conn(|conn| {
                        conn.prepare_cached("SELECT * FROM transactions WHERE transaction_id = ?")?
                            .query_row([transaction_id], |row| {
                                Ok(Transaction {
                                    id: row.get(0)?,
                                    transaction_address: row.get(1)?,
                                    timestamp: row.get(2)?,
                                    state_version: row.get(3)?,
                                    balance_changes,
                                    message: row.get(4)?,
                                })
                            })
                    })
                    .await
                    .ok()
            })
            .collect()
            .await;

        Ok(transactions)
    }

    pub async fn get_balance_changes_for_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<HashMap<TransactionId, Vec<BalanceChange>>, async_sqlite::Error> {
        type ReturnType = HashMap<TransactionId, Vec<BalanceChange>>;
        self.client
            .conn(|conn| {
                conn.prepare_cached("SELECT * FROM balance_changes WHERE account_address = ?")?
                    .query_map([account_address], |row| {
                        let transaction_id: TransactionId = row.get(5)?;
                        let balance_change = BalanceChange {
                            id: row.get(0)?,
                            account: row.get(1)?,
                            resource: row.get(2)?,
                            nfids: row.get(3)?,
                            amount: row.get(4)?,
                        };
                        Ok((transaction_id, balance_change))
                    })?
                    .fold::<Result<ReturnType, rusqlite::Error>, _>(
                        Err(rusqlite::Error::QueryReturnedNoRows),
                        |acc, result| match result {
                            Ok((transaction_id, balance_change)) => match acc {
                                Ok(mut map) => {
                                    map.entry(transaction_id)
                                        .or_insert(Vec::new())
                                        .push(balance_change);
                                    Ok(map)
                                }
                                Err(_) => {
                                    let mut map = HashMap::new();
                                    map.insert(transaction_id, vec![balance_change]);
                                    Ok(map)
                                }
                            },
                            Err(err) => acc.map_err(|_| err),
                        },
                    )
            })
            .await
    }

    pub async fn get_all_transactions<T>(&self) -> Result<T, async_sqlite::Error>
    where
        T: FromAsyncIterator<Transaction> + Send + 'static,
    {
        let transactions: Vec<Transaction> = self
            .client
            .conn(|conn| {
                conn.prepare_cached("SELECT * FROM transactions")?
                    .query_map([], |row| {
                        Ok(Transaction {
                            id: row.get(0)?,
                            transaction_address: row.get(1)?,
                            timestamp: row.get(2)?,
                            state_version: row.get(3)?,
                            balance_changes: Vec::new(),
                            message: row.get(4)?,
                        })
                    })?
                    .collect::<Result<_, _>>()
            })
            .await?;

        let transactions = transactions
            .into_aiter()
            .afilter_map(|mut transaction| async move {
                let balance_changes = self
                    .get_balance_changes_for_transaction(transaction.id.clone())
                    .await
                    .ok()?;
                transaction.balance_changes = balance_changes;
                Some(transaction)
            })
            .collect()
            .await;

        Ok(transactions)
    }

    pub async fn get_balance_changes_for_transaction<T>(
        &self,
        transaction_id: TransactionId,
    ) -> Result<T, async_sqlite::Error>
    where
        T: FromIterator<BalanceChange> + Send + 'static,
    {
        self.client
            .conn(|conn| {
                conn.prepare_cached("SELECT * FROM balance_changes WHERE transaction_id = ?")?
                    .query_map([transaction_id], |row| {
                        Ok(BalanceChange {
                            id: row.get(0)?,
                            account: row.get(1)?,
                            resource: row.get(2)?,
                            nfids: row.get(3)?,
                            amount: row.get(4)?,
                        })
                    })?
                    .collect()
            })
            .await
    }
}

// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use bip39::Mnemonic;

//     use types::crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair};
//     use types::Network;

//     use super::*;

//     #[test]
//     fn get_accounts() {
//         let db = Db::new_in_memory();
//         db.create_table_accounts()
//             .expect("Unable to create table 'accounts'");

//         let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
//         let (keypair, derivation_path) = Ed25519KeyPair::new(
//             &mnemonic,
//             None,
//             0,
//             Network::Mainnet,
//             Bip32Entity::Account,
//             Bip32KeyKind::TransactionSigning,
//         );

//         let pub_key = keypair.radixdlt_public_key();
//         let address = keypair.bech32_address();
//         let address =
//             AccountAddress::from_str(address.as_str()).expect("Unable to parse account address");

//         let account = Account::new(
//             0,
//             "test".to_owned(),
//             Network::Mainnet,
//             derivation_path,
//             address,
//             pub_key,
//         );

//         db.upsert_account(&account)
//             .expect("Unable to create account");

//         let mut accounts = db
//             .get_accounts_map::<HashMap<AccountAddress, Account>>()
//             .expect("Unable to get accounts map");
//         let retrieved_account = accounts
//             .remove(&account.address)
//             .expect("Account not found");

//         assert_eq!(account, retrieved_account);
//     }
// }
