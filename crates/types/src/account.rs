
use scrypto::prelude::radix_engine_common::crypto::Ed25519PublicKey;

use super::{entity_account::Settings, AccountAddress, Network};

pub struct AccountCollection(pub Vec<Account>);

#[derive(Debug, Clone, Eq)]
pub struct Account {
    pub address: AccountAddress,
    pub id: usize,
    pub name: String,
    pub network: Network,
    //Stored the derivation path as bytes for serialization in the database
    pub derivation_path: [u8; 24],
    pub public_key: Ed25519PublicKey,
    pub hidden: bool,
    pub settings: Settings,
}

impl Account {
    pub fn new(
        id: usize,
        name: String,
        network: Network,
        derivation_path: [u32; 6],
        address: AccountAddress,
        public_key: Ed25519PublicKey,
    ) -> Self {
        let mut path = [0u8; 24];

        for i in 0..derivation_path.len() {
            let bytes = derivation_path[i].to_be_bytes();
            path[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }

        Self {
            id,
            name,
            network,
            derivation_path: path,
            address,
            public_key,
            hidden: false,
            settings: Settings::default(),
        }
    }

    pub fn none() -> Self {
        let pub_key = [0u8; Ed25519PublicKey::LENGTH];
        Self {
            id: 0,
            name: "No Account".to_owned(),
            network: Network::Mainnet,
            derivation_path: [0u8; 24],
            address: AccountAddress::empty(),
            public_key: Ed25519PublicKey::try_from(pub_key.as_slice())
                .expect("Can not create public key from slice, module Account"),
            hidden: false,
            settings: Settings::default(),
        }
    }

    //Todo: implement test for this
    pub fn derivation_path(&self) -> [u32; 6] {
        let mut path = [0u32; 6];

        for i in 0..path.len() {
            let bytes = self.derivation_path[i * 4..i * 4 + 4]
                .try_into()
                .unwrap_or_else(|_| unreachable!("Failed to convert derivation path from bytes"));
            path[i] = u32::from_be_bytes(bytes);
        }

        path
    }
}

impl ToString for Account {
    fn to_string(&self) -> String {
        format!("{}  {}", self.name, self.address.as_str())
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
    }
}

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for Account {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}



#[cfg(test)]
mod test {
    use scrypto::crypto::Ed25519PublicKey;

    use super::AccountAddress;

    use super::Account;

    use rand;

    #[test]
    fn test_derivation_path() {
        let derivation_path:[u32;6] = [3244,1022,1,525,1460,1];

        let public_key = Ed25519PublicKey([0;Ed25519PublicKey::LENGTH]);

        let account = Account::new(0, "test".to_owned(), super::Network::Mainnet, derivation_path.clone(), AccountAddress::empty(), public_key);

        let reconstructed_derivation_path = account.derivation_path();

        assert_eq!(derivation_path, reconstructed_derivation_path);
    }

    #[test]
    fn test_derivation_path_random() {
        for _ in 0..1000 {
            let derivation_path:[u32;6] = [rand::random(),rand::random(),rand::random(),rand::random(),rand::random(),rand::random()];

            let public_key = Ed25519PublicKey([0;Ed25519PublicKey::LENGTH]);

            let account = Account::new(0, "test".to_owned(), super::Network::Mainnet, derivation_path.clone(), AccountAddress::empty(), public_key); 

            let reconstructed_derivation_path = account.derivation_path();

            assert_eq!(derivation_path, reconstructed_derivation_path);
        }
    }
}