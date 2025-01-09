use super::derivation_path_indexes::{BIP32_ENTITY_ACCOUNT, BIP32_ENTITY_IDENTITY};


#[derive(Debug)]
pub enum Bip32Entity {
    Account,
    Identity,
}

impl Bip32Entity {
    pub fn path_index(&self) -> u32 {
        match self {
            Bip32Entity::Account => BIP32_ENTITY_ACCOUNT,
            Bip32Entity::Identity => BIP32_ENTITY_IDENTITY,
        }
    }
}