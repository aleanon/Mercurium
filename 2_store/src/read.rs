use crate::{DbError, IconCache};

use super::Db;
use async_sqlite::rusqlite;
use asynciter::{AsyncIterator, FromAsyncIterator, IntoAsyncIterator};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use types::{
    address::{AccountAddress, Address, ResourceAddress},
    assets::{FungibleAsset, NonFungibleAsset},
    crypto::HashedPassword,
    Account, BalanceChange, Ed25519PublicKey, Resource, Transaction, TransactionId,
};

impl Db {
    pub async fn get_db_password_hash(&self) -> Result<HashedPassword, DbError> {
        Ok(self
            .client
            .conn(|conn| {
                conn.prepare_cached("SELECT password FROM password_hash WHERE id = 1")?
                    .query_row([], |row| Ok(row.get(0)?))
            })
            .await?)
    }

    pub async fn get_account(&self, account_address: AccountAddress) -> Result<Account, DbError> {
        Ok(self
            .client
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
            .await?)
    }

    pub async fn get_account_addresses<T>(&self) -> Result<T, DbError>
    where
        T: FromIterator<AccountAddress> + Send + 'static,
    {
        Ok(self
            .client
            .conn(|conn| {
                conn.prepare_cached("SELECT address FROM accounts")?
                    .query_map([], |row| Ok(row.get(0)?))?
                    .collect()
            })
            .await?)
    }

    pub async fn get_accounts<T>(&self) -> Result<T, DbError>
    where
        T: FromIterator<Account> + Send + 'static,
    {
        Ok(self
            .client
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
            .await?)
    }

    pub async fn get_all_fungible_assets_per_account<T, U>(&self) -> Result<T, DbError>
    where
        T: FromIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<FungibleAsset> + Send + 'static,
    {
        let accounts = self.get_account_addresses::<Vec<_>>().await?;

        Ok(self
            .client
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
            .await?)
    }

    pub async fn get_fungible_assets_for_accounts<T, U>(
        &self,
        account_addresses: Vec<AccountAddress>,
    ) -> Result<T, DbError>
    where
        T: FromIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<FungibleAsset> + Send + 'static,
    {
        Ok(self
            .client
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
            .await?)
    }

    pub async fn get_all_non_fungible_assets_per_account<T, U>(&self) -> Result<T, DbError>
    where
        T: FromIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        let accounts = self.get_account_addresses::<Vec<_>>().await?;

        Ok(self
            .client
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
            .await?)
    }

    pub async fn get_non_fungible_assets_for_accounts<T, U>(
        &self,
        account_addresses: Vec<AccountAddress>,
    ) -> Result<T, DbError>
    where
        T: FromIterator<(AccountAddress, U)> + Send + 'static,
        U: FromIterator<NonFungibleAsset> + Send + 'static,
    {
        Ok(self
            .client
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
            .await?)
    }

    pub async fn get_all_resources<T>(&self) -> Result<T, DbError>
    where
        T: FromIterator<Resource> + Send + 'static,
    {
        Ok(self
            .client
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
            .await?)
    }

    pub async fn get_last_transaction_for_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<Transaction, DbError> {
        let mut transactions: BTreeSet<Transaction> =
            self.get_transactions_for_account(account_address).await?;
        transactions
            .pop_last()
            .ok_or(async_sqlite::Error::Rusqlite(rusqlite::Error::QueryReturnedNoRows).into())
    }

    pub async fn get_transactions_for_account<T>(
        &self,
        account_address: AccountAddress,
    ) -> Result<T, DbError>
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
    ) -> Result<HashMap<TransactionId, Vec<BalanceChange>>, DbError> {
        type ReturnType = HashMap<TransactionId, Vec<BalanceChange>>;
        Ok(self
            .client
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
            .await?)
    }

    pub async fn get_all_transactions<T>(&self) -> Result<T, DbError>
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
    ) -> Result<T, DbError>
    where
        T: FromIterator<BalanceChange> + Send + 'static,
    {
        Ok(self
            .client
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
            .await?)
    }
}

impl IconCache {
    pub async fn get_all_resource_icons(
        &self,
    ) -> Result<HashMap<ResourceAddress, Vec<u8>>, DbError> {
        let result = self
            .client
            .conn(|conn| {
                conn.prepare_cached("SELECT * FROM resource_images")?
                    .query_map([], |row| {
                        let resource_address: ResourceAddress = row.get(0)?;
                        let image_data: Vec<u8> = row.get(1)?;
                        Ok((resource_address, image_data))
                    })?
                    .collect::<Result<HashMap<ResourceAddress, Vec<u8>>, rusqlite::Error>>()
            })
            .await?;
        Ok(result)
    }

    pub async fn get_resource_icon(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<(ResourceAddress, Vec<u8>), DbError> {
        Ok(self
            .client
            .conn(move |conn| {
                conn.query_row(
                    "SELECT * FROM resource_images WHERE resource_address = ?",
                    [resource_address],
                    |row| {
                        let resource_address: ResourceAddress = row.get(0)?;
                        let image_data: Vec<u8> = row.get(1)?;
                        Ok((resource_address, image_data))
                    },
                )
            })
            .await?)
    }

    pub async fn get_all_nft_images_for_resource(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<(ResourceAddress, BTreeMap<String, Vec<u8>>), DbError> {
        let resource_address_params = resource_address.clone();
        let btree_map = self
            .client
            .conn(|conn| {
                conn.prepare_cached("SELECT * FROM nft_images WHERE resource_address = ?")?
                    .query_map([resource_address_params], |row| {
                        let mut nfid: String = row.get(0)?;
                        let _ = nfid.split_off(nfid.len() - ResourceAddress::CHECKSUM_LENGTH);
                        let image_data: Vec<u8> = row.get(1)?;
                        Ok((nfid, image_data))
                    })?
                    .collect::<Result<BTreeMap<String, Vec<u8>>, rusqlite::Error>>()
            })
            .await?;
        Ok((resource_address, btree_map))
    }

    pub async fn get_nft_image(
        &self,
        resource_address: ResourceAddress,
        nfid: String,
    ) -> Result<(ResourceAddress, String, Vec<u8>), DbError> {
        Ok(self
            .client
            .conn(move |conn| {
                let mut nfid_param = nfid.clone();
                nfid_param.push_str(resource_address.checksum_as_str());

                conn.query_row(
                    "SELECT * FROM nft_images WHERE nfid =?",
                    [nfid_param],
                    |row| {
                        let image_data: Vec<u8> = row.get(1)?;
                        Ok((resource_address, nfid, image_data))
                    },
                )
            })
            .await?)
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
