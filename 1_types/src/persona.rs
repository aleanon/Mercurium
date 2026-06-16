use deps::*;

use async_sqlite::rusqlite;
use scrypto::crypto::Ed25519PublicKey;
use serde::{Deserialize, Serialize};

use crate::Network;

/// A Radix persona — an on-ledger *identity* entity the user presents to dApps via Radix
/// Connect. A persona owns a derived identity key (used for ROLA authentication) and a set of
/// user-controlled [`PersonaData`] fields the user may choose to share with a dApp.
///
/// The `identity_address` is stored as the bech32m string (`identity_...`). A fully validated
/// `IdentityAddress` newtype (mirroring `AccountAddress`) is a follow-up; for now the address is
/// produced by deriving it from the identity key, so it is always well-formed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Persona {
    pub id: i64,
    pub label: String,
    pub network: Network,
    /// bech32m `identity_...` address.
    pub identity_address: String,
    pub public_key: Ed25519PublicKey,
    /// Derivation path stored as bytes, identical layout to `Account::derivation_path`.
    pub derivation_path: [u8; 24],
    pub persona_data: PersonaData,
    pub hidden: bool,
}

impl Persona {
    pub fn new(
        id: i64,
        label: String,
        network: Network,
        identity_address: String,
        public_key: Ed25519PublicKey,
        derivation_path: [u32; 6],
        persona_data: PersonaData,
    ) -> Self {
        let mut path = [0u8; 24];
        for i in 0..derivation_path.len() {
            path[i * 4..i * 4 + 4].copy_from_slice(&derivation_path[i].to_be_bytes());
        }

        Self {
            id,
            label,
            network,
            identity_address,
            public_key,
            derivation_path: path,
            persona_data,
            hidden: false,
        }
    }

    pub fn derivation_path(&self) -> [u32; 6] {
        let mut path = [0u32; 6];
        for i in 0..path.len() {
            let bytes = self.derivation_path[i * 4..i * 4 + 4]
                .try_into()
                .expect("derivation path slice is 4 bytes");
            path[i] = u32::from_be_bytes(bytes);
        }
        path
    }

    pub fn derivation_index(&self) -> u32 {
        let bytes = self.derivation_path[20..]
            .try_into()
            .expect("derivation index slice is 4 bytes");
        u32::from_be_bytes(bytes)
    }
}

/// User-controlled data a persona may share with dApps (CAP-21 persona data).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonaData {
    pub name: Option<PersonaName>,
    pub email_addresses: Vec<String>,
    pub phone_numbers: Vec<String>,
}

impl rusqlite::types::FromSql for PersonaData {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Blob(blob) => Ok(serde_json::from_slice(blob)
                .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::types::ToSql for PersonaData {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Blob(
                serde_json::to_vec(self)
                    .map_err(|err| rusqlite::types::FromSqlError::Other(Box::new(err)))?,
            ),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonaName {
    pub given: String,
    pub family: String,
    pub variant: NameVariant,
}

/// Whether the persona's name is displayed family-name-first ("Eastern") or
/// given-name-first ("Western"), matching the wallet's persona-data model.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum NameVariant {
    #[default]
    Western,
    Eastern,
}
