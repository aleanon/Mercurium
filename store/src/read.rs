use std::collections::{BTreeMap, HashMap};

use rusqlite::OptionalExtension;

use super::{AsyncDb, Db};
use types::{
    account::Account,  AccountAddress, EntityAccount, Fungible, Fungibles,
    NonFungible, NonFungibles, ResourceAddress, Ed25519PublicKey,
};
use anyhow::Result;

impl Db {
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
                    current_supply: row.get(5)?,
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
                    current_supply: row.get(5)?,
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
    ) -> Result<Option<NonFungibles>, rusqlite::Error> {
        match self
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
            .collect::<Result<Vec<NonFungible>, rusqlite::Error>>()
            .optional()?
        {
            Some(vec) => Ok(Some(vec.into())),
            None => Ok(None),
        }
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
                            current_supply: row.get(5)?,
                            description: row.get(6)?,
                            last_updated_at_state_version: row.get(7)?,
                            metadata: row.get(8)?,
                        })
                    })
                    .optional()
            })
            .await
    }

    pub async fn get_all_fungibles(
        &self,
    ) -> Result<HashMap<AccountAddress, Fungible>, rusqlite::Error> {
        self.connection
            .call_unwrap(|conn| {
                conn.prepare_cached("SELECT * FROM fungibles")?
                    .query_map([], |row| {
                        let account: AccountAddress = row.get(9)?;
                        let fungible = Fungible {
                            address: row.get(0)?,
                            name: row.get(1)?,
                            symbol: row.get(2)?,
                            icon: row.get(3)?,
                            amount: row.get(4)?,
                            current_supply: row.get(5)?,
                            description: row.get(6)?,
                            last_updated_at_state_version: row.get(7)?,
                            metadata: row.get(8)?,
                        };
                        Ok((account, fungible))
                    })?
                    .collect()
            })
            .await
    }

    pub async fn get_fungibles_by_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<Fungibles, rusqlite::Error> {
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
                            current_supply: row.get(5)?,
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

    // pub async fn get_all_non_fungibles(&self) -> Result<HashMap<AccountAddress, NonFungible>, rusqlite::Error> {
    //     self.connection.call_unwrap(|conn| {
    //         conn.prepare_cached("SELECT * FROM non_fungibles")?
    //         .query_map([], |row| {
    //             let account:AccountAddress = row.get(8)?;
    //             let non_fungible = NonFungible {
    //                 address: row.get(0)?,
    //                 name: row.get(1)?,
    //                 symbol: row.get(2)?,
    //                 icon: row.get(3)?,
    //                 description: row.get(4)?,
    //                 nfids: row.get(5)?,
    //                 last_updated_at_state_version: row.get(6)?,
    //                 metadata: row.get(7)?,
    //             };
    //             Ok((account, non_fungible))
    //         })?
    //         .collect()
    //     }).await
    // }

    pub async fn get_non_fungibles_by_account(
        &self,
        account_address: AccountAddress,
    ) -> Result<Option<NonFungibles>, rusqlite::Error> {
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
                    .optional()
            })
            .await
    }

    pub async fn get_entityaccounts(&self) -> Result<Vec<EntityAccount>, rusqlite::Error> {
        let accounts = self.get_accounts_map().await?;
        let mut entity_accounts = vec![];

        for (account_address, account) in accounts.into_iter() {
            let fungibles = self
                .get_fungibles_by_account(account_address.clone())
                .await?;
            let non_fungibles = self.get_non_fungibles_by_account(account_address).await?;
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

    use handles::crypto::ed25519::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair};
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

        db.update_account(&account)
            .expect("Unable to create account");

        let mut accounts = db.get_accounts_map().expect("Unable to get accounts map");
        let retrieved_account = accounts
            .remove(&account.address)
            .expect("Account not found");

        assert_eq!(account, retrieved_account);
    }
}
