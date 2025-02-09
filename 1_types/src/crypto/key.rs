
use super::salt::Salt;
use ring::pbkdf2::{self, PBKDF2_HMAC_SHA256};
use std::{fmt::Debug, marker::PhantomData, num::NonZeroU32};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub trait KeyType {
    const LENGTH: usize;
    const ITERATIONS: NonZeroU32;
}

const KEY_LENGTH: usize = 32;

#[derive(Clone, ZeroizeOnDrop, Zeroize)]
pub struct Key<T> 
    where 
        T: KeyType,
        
{
    inner: [u8; KEY_LENGTH],
    _marker: std::marker::PhantomData<T>,
}


impl<T> Key<T> 
    where 
        T: KeyType,
{
    pub const LENGTH: usize = 32;
    // //Iteration counts needs to be > 0 else the program will panic
    // const DB_KEY_ITERATIONS: u32 = 200000;
    // const MNEMONIC_KEY_ITERATIONS: u32 = 2000000;


    pub fn new(source: &str, salt: &Salt) -> Self {
        // Change to generic array length through the KeyType trait when the functionality is stable
        let mut key = [0u8;KEY_LENGTH];

        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            T::ITERATIONS,
            salt.as_bytes(),
            source.as_bytes(),
            &mut key,
        );

        Self{inner: key, _marker: PhantomData}
    }

    // pub fn db_encryption_key(salt: &Salt, password: &Password) -> Self {
    //     let mut key = [0u8; KeyType::LENGTH];
    //     let iterations = NonZeroU32::new(Self::DB_KEY_ITERATIONS)
    //         .unwrap_unreachable(debug_info!("Attempted to create NonZeroU32 from a 0 value"));

    //     pbkdf2::derive(
    //         PBKDF2_HMAC_SHA256,
    //         iterations,
    //         salt.as_bytes(),
    //         password.as_str().as_bytes(),
    //         &mut key,
    //     );

    //     Self(key)
    // }

    // pub fn mnemonic_encryption_key(salt: &Salt, password: &Password) -> Self {
    //     let mut key = [0u8; Self::LENGTH];
    //     let iterations = NonZeroU32::new(Self::MNEMONIC_KEY_ITERATIONS)
    //         .unwrap_unreachable(debug_info!("Attempted to create NonZeroU32 from a 0 value"));

    //     pbkdf2::derive(
    //         PBKDF2_HMAC_SHA256,
    //         iterations,
    //         salt.as_bytes(),
    //         password.as_str().as_bytes(),
    //         &mut key,
    //     );

    //     Self(key)
    // }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    // pub fn into_database_key(self) -> DataBaseKey {
    //     DataBaseKey::from_key(self)
    // }

    pub fn into_inner(mut self) -> [u8; KEY_LENGTH] {
        std::mem::take(&mut self.inner)
    }
}

impl<T> Default for Key<T> 
    where 
        T: KeyType,
{
    fn default() -> Self {
        Self {
            inner: [0; KEY_LENGTH],
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for Key<T> 
    where 
        T: KeyType,
    {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Key<{:?}>", std::any::type_name::<T>())
    }
}


// A Hexadecimal representation of `Key` with a prefix (x') and end (') so it can be formatted as text and passed as a hexadecimal key to the database
// #[derive(Debug, ZeroizeOnDrop, Clone)]
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
