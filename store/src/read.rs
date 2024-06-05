use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Context;
use rusqlite::OptionalExtension;

use super::{AsyncDb, Db};
use types::{
    account::Account,
    assets::{FungibleAsset, NonFungibleAsset},
    fungibles, non_fungibles,
    resource::Resource,
    transaction::{BalanceChange, TransactionId},
    AccountAddress, Ed25519PublicKey, EntityAccount, Fungible, Fungibles, NonFungible,
    NonFungibles, ResourceAddress, Transaction,
};

impl Db {
    pub fn get_fungible_assets_for_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<HashMap<ResourceAddress, FungibleAsset>, rusqlite::Error> {
        self.connection
            .prepare_cached("SELECT * FROM fungible_assets WHERE account_address = ?")?
            .query_map([account_address.as_str()], |row| {
                let fungible_asset = FungibleAsset {
                    id: row.get(0)?,
                    resource_address: row.get(1)?,
                    amount: row.get(2)?,
                    last_updated: row.get(3)?,
                };
                Ok((fungible_asset.resource_address.clone(), fungible_asset))
            })?
            .collect()
    }

    pub fn get_non_fungible_assets_for_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<HashMap<ResourceAddress, NonFungibleAsset>, rusqlite::Error> {
        self.connection
            .prepare_cached("SELECT * FROM fungible_assets WHERE account_address = ?")?
            .query_map([account_address.as_str()], |row| {
                let fungible_asset = NonFungibleAsset {
                    id: row.get(0)?,
                    resource_address: row.get(1)?,
                    nfids: row.get(2)?,
                    last_updated: row.get(3)?,
                };
                Ok((fungible_asset.resource_address.clone(), fungible_asset))
            })?
            .collect()
    }

    pub fn get_fungible_assets_for_accounts(
        &self,
        account_addresses: &[AccountAddress],
    ) -> Result<HashMap<AccountAddress, HashMap<ResourceAddress, FungibleAsset>>, rusqlite::Error>
    {
        let mut fungibles_by_account = HashMap::new();

        let mut stmt = self
            .connection
            .prepare_cached("SELECT * FROM fungible_assets WHERE account_address = ?")?;

        for account_address in account_addresses {
            let fungible_assets = stmt
                .query_map([account_address.as_str()], |row| {
                    let fungible_asset = FungibleAsset {
                        id: row.get(0)?,
                        resource_address: row.get(1)?,
                        amount: row.get(2)?,
                        last_updated: row.get(3)?,
                    };

                    Ok((fungible_asset.resource_address.clone(), fungible_asset))
                })?
                .filter_map(|result| result.ok())
                .collect();

            fungibles_by_account.insert(account_address.clone(), fungible_assets);
        }

        Ok(fungibles_by_account)
    }

    pub fn get_non_fungible_assets_for_accounts(
        &self,
        account_addresses: &[AccountAddress],
    ) -> Result<HashMap<AccountAddress, HashMap<ResourceAddress, NonFungibleAsset>>, rusqlite::Error>
    {
        let mut fungibles_by_account = HashMap::new();

        let mut stmt = self
            .connection
            .prepare_cached("SELECT * FROM fungible_assets WHERE account_address = ?")?;

        for account_address in account_addresses {
            let fungible_assets = stmt
                .query_map([account_address.as_str()], |row| {
                    let non_fungible_asset = NonFungibleAsset {
                        id: row.get(0)?,
                        resource_address: row.get(1)?,
                        nfids: row.get(2)?,
                        last_updated: row.get(3)?,
                    };

                    Ok((
                        non_fungible_asset.resource_address.clone(),
                        non_fungible_asset,
                    ))
                })?
                .filter_map(|result| result.ok())
                .collect();

            fungibles_by_account.insert(account_address.clone(), fungible_assets);
        }

        Ok(fungibles_by_account)
    }

    pub fn get_all_resources(&self) -> Result<HashMap<ResourceAddress, Resource>, rusqlite::Error> {
        Ok(self
            .connection
            .prepare_cached("SELECT * FROM resources")?
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
                Ok((resource.address.clone(), resource))
            })?
            .filter_map(|result| result.ok())
            .collect())
    }

    pub fn get_fungible(
        &self,
        resource_address: &ResourceAddress,
    ) -> Result<Option<Fungible>, rusqlite::Error> {
        Ok(self
            .connection
            .prepare_cached("SELECT * FROM fungibles WHERE address = ?")?
            .query_row([resource_address], |row| {
                Ok(Fungible {
                    address: row.get(0)?,
                    name: row.get(1)?,
                    symbol: row.get(2)?,
                    icon: row.get(3)?,
                    amount: row.get(4)?,
                    total_supply: row.get(5)?,
                    description: row.get(6)?,
                    last_updated_at_state_version: row.get(7)?,
                    metadata: row.get(8)?,
                })
            })
            .optional()?)
    }

    pub fn get_fungibles_by_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Fungibles, rusqlite::Error> {
        Ok(self
            .connection
            .prepare_cached("SELECT * FROM fungibles WHERE account_address = ?")?
            .query_map([account_address], |row| {
                Ok(Fungible {
                    address: row.get(0)?,
                    name: row.get(1)?,
                    symbol: row.get(2)?,
                    icon: row.get(3)?,
                    amount: row.get(4)?,
                    total_supply: row.get(5)?,
                    description: row.get(6)?,
                    last_updated_at_state_version: row.get(7)?,
                    metadata: row.get(8)?,
                })
            })?
            .collect::<Result<Fungibles, rusqlite::Error>>()?)
    }

    pub fn get_non_fungibles_by_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<NonFungibles, rusqlite::Error> {
        Ok(self
            .connection
            .prepare_cached("SELECT * FROM non_fungibles WHERE account_address = ?")?
            .query_map([account_address], |row| {
                Ok(NonFungible {
                    address: row.get(0)?,
                    name: row.get(1)?,
                    symbol: row.get(2)?,
                    icon: row.get(3)?,
                    description: row.get(4)?,
                    nfids: row.get(5)?,
                    last_updated_at_state_version: row.get(6)?,
                    metadata: row.get(7)?,
                })
            })?
            .collect::<Result<NonFungibles, rusqlite::Error>>()?)
    }

    pub fn get_entityaccounts(&self) -> Result<Vec<EntityAccount>, rusqlite::Error> {
        self.connection
            .prepare_cached("SELECT * FROM accounts")?
            .query_map([], |row| {
                let address: AccountAddress = row.get(0)?;
                let fungibles = self.get_fungibles_by_account(&address)?;
                let non_fungibles = self.get_non_fungibles_by_account(&address)?;
                Ok(EntityAccount {
                    address,
                    id: row.get(1)?,
                    name: row.get(2)?,
                    fungibles,
                    non_fungibles,
                    transactions: None,
                    settings: row.get(7)?,
                })
            })?
            .collect()
    }

    pub fn get_account_addresses(&self) -> Result<Vec<AccountAddress>, rusqlite::Error> {
        self.connection
            .prepare_cached("SELECT address FROM accounts")?
            .query_map([], |row| row.get(0))?
            .collect()
    }

    pub fn get_accounts_map(&self) -> Result<BTreeMap<AccountAddress, Account>, rusqlite::Error> {
        self.connection
            .prepare_cached("SELECT * FROM accounts")?
            .query_map([], |row| {
                let address: AccountAddress = row.get(0)?;
                let account = Account {
                    address: address.clone(),
                    id: row.get(1)?,
                    name: row.get(2)?,
                    network: row.get(3)?,
                    derivation_path: row.get(4)?,
                    public_key: Ed25519PublicKey(row.get(5)?),
                    hidden: row.get(6)?,
                    settings: row.get(7)?,
                };
                Ok((address, account))
            })?
            .collect()
    }

    pub fn get_transactions_for_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<BTreeSet<Transaction>, rusqlite::Error> {
        let balance_changes = self.get_balance_changes_for_account(account_address)?;

        balance_changes
            .into_iter()
            .map(|(transaction_id, balance_changes)| {
                self.connection
                    .prepare_cached("SELECT * FROM transactions WHERE transaction_id = ?")?
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
            .collect()
    }

    pub fn get_balance_changes_for_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<HashMap<TransactionId, Vec<BalanceChange>>, rusqlite::Error> {
        self.connection
            .prepare_cached("SELECT * FROM balance_changes WHERE account_address = ?")?
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
            .fold(
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
    }
}

impl AsyncDb {
    pub async fn get_fungible(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<Option<Fungible>, rusqlite::Error> {
        self.connection
            .call_unwrap(|conn| {
                conn.prepare_cached("SELECT * FROM fungibles WHERE address = ?")?
                    .query_row([resource_address], |row| {
                        Ok(Fungible {
                            address: row.get(0)?,
                            name: row.get(1)?,
                            symbol: row.get(2)?,
                            icon: row.get(3)?,
                            amount: row.get(4)?,
                            total_supply: row.get(5)?,
                            description: row.get(6)?,
                            last_updated_at_state_version: row.get(7)?,
                            metadata: row.get(8)?,
                        })
                    })
                    .optional()
            })
            .await
    }

    pub async fn get_fungibles_by_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Fungibles, rusqlite::Error> {
        let account_address = account_address.clone();
        let result = self
            .connection
            .call_unwrap(|conn| {
                conn.prepare_cached("SELECT * FROM fungibles WHERE account_address = ?")?
                    .query_map([account_address], |row| {
                        Ok(Fungible {
                            address: row.get(0)?,
                            name: row.get(1)?,
                            symbol: row.get(2)?,
                            icon: row.get(3)?,
                            amount: row.get(4)?,
                            total_supply: row.get(5)?,
                            description: row.get(6)?,
                            last_updated_at_state_version: row.get(7)?,
                            metadata: row.get(8)?,
                        })
                    })?
                    .collect::<Result<Fungibles, rusqlite::Error>>()
            })
            .await?;
        Ok(result)
    }

    pub async fn get_non_fungibles_by_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<NonFungibles, rusqlite::Error> {
        let account_address = account_address.clone();
        self.connection
            .call_unwrap(|conn| {
                conn.prepare_cached("SELECT * FROM non_fungibles WHERE account_address = ?")?
                    .query_map([account_address], |row| {
                        Ok(NonFungible {
                            address: row.get(0)?,
                            name: row.get(1)?,
                            symbol: row.get(2)?,
                            icon: row.get(3)?,
                            description: row.get(4)?,
                            nfids: row.get(5)?,
                            last_updated_at_state_version: row.get(6)?,
                            metadata: row.get(7)?,
                        })
                    })?
                    .collect::<Result<NonFungibles, rusqlite::Error>>()
            })
            .await
    }

    pub async fn get_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Account, rusqlite::Error> {
        let account_address = account_address.clone();
        self.connection
            .call_unwrap(|conn| {
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
                        })
                    })
            })
            .await
    }

    pub async fn get_entity_account(
        &self,
        account_address: &AccountAddress,
    ) -> Result<EntityAccount, rusqlite::Error> {
        let account = self.get_account(account_address).await?;
        let fungibles = self.get_fungibles_by_account(&account.address).await?;
        let non_fungibles = self.get_non_fungibles_by_account(&account.address).await?;
        Ok(EntityAccount {
            address: account.address,
            id: account.id,
            name: account.name,
            fungibles,
            non_fungibles,
            transactions: None,
            settings: account.settings,
        })
    }

    pub async fn get_entityaccounts(&self) -> Result<Vec<EntityAccount>, rusqlite::Error> {
        let accounts = self.get_accounts_map().await?;
        let mut entity_accounts = vec![];

        for (account_address, account) in accounts.into_iter() {
            let fungibles = self.get_fungibles_by_account(&account_address).await?;
            let non_fungibles = self.get_non_fungibles_by_account(&account_address).await?;
            let entity_account = EntityAccount {
                address: account.address,
                id: account.id,
                name: account.name,
                fungibles,
                non_fungibles,
                transactions: None,
                settings: account.settings,
            };

            entity_accounts.push(entity_account)
        }

        Ok(entity_accounts)
    }

    pub async fn get_account_addresses(&self) -> Result<Vec<AccountAddress>, rusqlite::Error> {
        self.connection
            .call_unwrap(|conn| {
                conn.prepare_cached("SELECT address FROM accounts")?
                    .query_map([], |row| Ok(row.get(0)?))?
                    .collect()
            })
            .await
    }

    pub async fn get_accounts_map(
        &self,
    ) -> Result<BTreeMap<AccountAddress, Account>, rusqlite::Error> {
        self.connection
            .call_unwrap(|conn| {
                conn.prepare_cached("SELECT * FROM accounts")?
                    .query_map([], |row| {
                        let address: AccountAddress = row.get(0)?;
                        let account = Account {
                            address: address.clone(),
                            id: row.get(1)?,
                            name: row.get(2)?,
                            network: row.get(3)?,
                            derivation_path: row.get(4)?,
                            public_key: Ed25519PublicKey(row.get(5)?),
                            hidden: row.get(6)?,
                            settings: row.get(7)?,
                        };
                        Ok((address, account))
                    })?
                    .collect()
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bip39::Mnemonic;
    use rusqlite::Connection;

    use types::crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair};
    use types::Network;

    use super::*;

    #[test]
    fn get_accounts() {
        let connection =
            Connection::open_in_memory().expect("Failed to create in memory connection");
        let mut db = Db::with_connection(connection);
        db.create_table_accounts()
            .expect("Unable to create table 'accounts'");

        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let (keypair, derivation_path) = Ed25519KeyPair::from_mnemonic(
            &mnemonic,
            0,
            Network::Mainnet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );

        let pub_key = keypair.radixdlt_public_key();
        let address = keypair.bech32_address();
        let address =
            AccountAddress::from_str(address.as_str()).expect("Unable to parse account address");

        let account = Account::new(
            0,
            "test".to_owned(),
            Network::Mainnet,
            derivation_path,
            address,
            pub_key,
        );

        db.upsert_account(&account)
            .expect("Unable to create account");

        let mut accounts = db.get_accounts_map().expect("Unable to get accounts map");
        let retrieved_account = accounts
            .remove(&account.address)
            .expect("Account not found");

        assert_eq!(account, retrieved_account);
    }
}
