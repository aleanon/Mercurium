use deps::*;
use deps::zeroize;

use types::{crypto::Key, debug_info, UnwrapUnreachable};
use zeroize::ZeroizeOnDrop;

use crate::DataBase;

#[derive(Debug, ZeroizeOnDrop, Clone)]
pub struct SqliteKey(Vec<u8>);

impl SqliteKey {
    const KEY_START: &[u8] = b"x\'";
    const KEY_END: u8 = b'\'';

    pub fn from_key(key: &Key<DataBase>) -> Self {
        let key_length = key.as_bytes().len() * 2;
        let mut db_hex_key = vec![0u8; key_length];

        for (i, byte) in key.as_bytes().iter().enumerate() {
            db_hex_key[i * 2] = Self::to_hex_digit(byte >> 4);
            db_hex_key[i * 2 + 1] = Self::to_hex_digit(byte & 0x0F);
        }

        db_hex_key.insert(0, Self::KEY_START[1]);
        db_hex_key.insert(0, Self::KEY_START[0]);
        db_hex_key.push(Self::KEY_END);

        Self(db_hex_key)
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0)
            .unwrap_unreachable(debug_info!("HexKey contained non utf8 bytes"))
    }

    fn to_hex_digit(n: u8) -> u8 {
        match n {
            0..=9 => b'0' + n,
            10..=15 => b'a' + (n - 10),
            _ => unreachable!(),
        }
    }

}


impl async_sqlite::rusqlite::ToSql for SqliteKey {
    fn to_sql(
        &self,
    ) -> Result<async_sqlite::rusqlite::types::ToSqlOutput, async_sqlite::rusqlite::Error> {
        Ok(async_sqlite::rusqlite::types::ToSqlOutput::Borrowed(
            async_sqlite::rusqlite::types::ValueRef::Text(&self.0),
        ))
    }
}

// A Hexadecimal representation of `Key` with a prefix (x') and end (') so it can be formatted as text and passed as a hexadecimal key to the database

// pub struct DataBaseKey([u8; Self::LENGTH]);

// impl DataBaseKey {
//     const KEY_LENGTH: usize = Key::LENGTH * 2;
//     const LENGTH: usize = Self::KEY_LENGTH + 3;
//     const DB_KEY_START: &[u8] = b"x\'";
//     const DB_KEY_END: &[u8] = b"\'";

//     pub fn dummy_one_use_key() -> Self {
//         let salt = Salt::new().expect("Failed to create random value for salt");
//         Password::from("dummy key").derive_db_encryption_key_from_salt(&salt)
//     }

//     pub fn from_key(key: Key) -> Self {
//         let mut hex_key = [0u8; Self::KEY_LENGTH];
//         for (i, byte) in key.as_bytes().iter().enumerate() {
//             hex_key[i * 2] = Self::to_hex_digit(byte >> 4);
//             hex_key[i * 2 + 1] = Self::to_hex_digit(byte & 0x0F);
//         }
//         let mut db_hex_key = [b' '; Self::LENGTH];
//         db_hex_key[..Self::DB_KEY_START.len()].copy_from_slice(Self::DB_KEY_START);
//         db_hex_key[Self::DB_KEY_START.len()..Self::KEY_LENGTH + Self::DB_KEY_START.len()]
//             .copy_from_slice(&hex_key);
//         db_hex_key[Self::LENGTH - Self::DB_KEY_END.len()..].copy_from_slice(Self::DB_KEY_END);

//         Self(db_hex_key)
//     }

//     pub fn as_str(&self) -> &str {
//         std::str::from_utf8(&self.0)
//             .unwrap_unreachable(debug_info!("HexKey contained non utf8 bytes"))
//     }

//     fn to_hex_digit(n: u8) -> u8 {
//         match n {
//             0..=9 => b'0' + n,
//             10..=15 => b'a' + (n - 10),
//             _ => unreachable!(),
//         }
//     }
// }

// impl async_sqlite::rusqlite::ToSql for Key<DataBaseKey> {
//     fn to_sql(
//         &self,
//     ) -> Result<async_sqlite::rusqlite::types::ToSqlOutput, async_sqlite::rusqlite::Error> {
//         Ok(async_sqlite::rusqlite::types::ToSqlOutput::Borrowed(
//             async_sqlite::rusqlite::types::ValueRef::Text(&self.inner),
//         ))
//     }
// }
