use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable};

use super::{password::Password, salt::Salt};
use ring::pbkdf2::{self, PBKDF2_HMAC_SHA256};
use std::num::NonZeroU32;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, ZeroizeOnDrop, Zeroize)]
// Represents a 256bit key
pub struct Key([u8; Self::LENGTH]);

impl Key {
    pub const LENGTH: usize = 32;
    //Iteration counts needs to be > 0 else the program will panic
    const DB_KEY_ITERATIONS: u32 = 200000;
    #[cfg(not(debug_assertions))]
    const MNEMONIC_KEY_ITERATIONS: u32 = 2000000;
    #[cfg(debug_assertions)]
    const MNEMONIC_KEY_ITERATIONS: u32 = 10000;

    pub fn new(password: &str, salt: &Salt, iterations: NonZeroU32) -> Self {
        let mut key = [0u8; Self::LENGTH];

        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            iterations,
            salt.as_bytes(),
            password.as_bytes(),
            &mut key,
        );

        Self(key)
    }

    pub fn db_encryption_key(salt: &Salt, password: &Password) -> Self {
        let mut key = [0u8; Self::LENGTH];
        let iterations = NonZeroU32::new(Self::DB_KEY_ITERATIONS)
            .unwrap_unreachable(debug_info!("Attempted to create NonZeroU32 from a 0 value"));

        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            iterations,
            salt.as_bytes(),
            password.as_str().as_bytes(),
            &mut key,
        );

        Self(key)
    }

    pub fn mnemonic_encryption_key(salt: &Salt, password: &Password) -> Self {
        let mut key = [0u8; Self::LENGTH];
        let iterations = NonZeroU32::new(Self::MNEMONIC_KEY_ITERATIONS)
            .unwrap_unreachable(debug_info!("Attempted to create NonZeroU32 from a 0 value"));

        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            iterations,
            salt.as_bytes(),
            password.as_str().as_bytes(),
            &mut key,
        );

        Self(key)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_database_key(self) -> DataBaseKey {
        DataBaseKey::from_key(self)
    }

    pub fn to_inner(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

/// A Hexadecimal representation of `Key` so it can be formatted as text and passed as key to the database
#[derive(Debug, ZeroizeOnDrop, Clone)]
pub struct DataBaseKey([u8; Self::LENGTH]);

impl DataBaseKey {
    const LENGTH: usize = Key::LENGTH * 2;
    const PRAGMA_END: &'static [u8] = b"'\"";
    const PRAGMA_KEY_START: &'static [u8] = b"PRAGMA key = \"x'";
    const PRAGMA_REKEY_START: &'static [u8] = b"PRAGMA rekey = \"x'";
    const PRAGMA_KEY_AND_ARGUMENT_LENGTH: usize =
        Self::LENGTH + Self::PRAGMA_KEY_START.len() + Self::PRAGMA_END.len();
    const PRAGMA_REKEY_AND_ARGUMENT_LENGTH: usize =
        Self::LENGTH + Self::PRAGMA_REKEY_START.len() + Self::PRAGMA_END.len();

    pub fn from_key(key: Key) -> Self {
        let mut hex_key = [0u8; Self::LENGTH];
        for (i, byte) in key.as_bytes().iter().enumerate() {
            hex_key[i * 2] = Self::to_hex_digit(byte >> 4);
            hex_key[i * 2 + 1] = Self::to_hex_digit(byte & 0x0F);
        }
        Self(hex_key)
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0)
            .unwrap_unreachable(debug_info!("HexKey contained non utf8 bytes"))
    }

    pub fn set_key_statement(&self) -> [u8; Self::PRAGMA_KEY_AND_ARGUMENT_LENGTH] {
        let mut statement = [b' '; Self::PRAGMA_KEY_AND_ARGUMENT_LENGTH];
        statement[..Self::PRAGMA_KEY_START.len()].copy_from_slice(Self::PRAGMA_KEY_START);
        statement[Self::PRAGMA_KEY_START.len()
            ..Self::PRAGMA_KEY_AND_ARGUMENT_LENGTH - Self::PRAGMA_END.len()]
            .copy_from_slice(&self.0);
        statement[Self::PRAGMA_KEY_AND_ARGUMENT_LENGTH - Self::PRAGMA_END.len()..]
            .copy_from_slice(Self::PRAGMA_END);
        statement
    }

    pub fn rekey_statement(&self) -> [u8; Self::PRAGMA_REKEY_AND_ARGUMENT_LENGTH] {
        let mut statement = [b' '; Self::PRAGMA_REKEY_AND_ARGUMENT_LENGTH];
        statement[..Self::PRAGMA_REKEY_START.len()].copy_from_slice(Self::PRAGMA_REKEY_START);
        statement[Self::PRAGMA_REKEY_START.len()
            ..Self::PRAGMA_REKEY_AND_ARGUMENT_LENGTH - Self::PRAGMA_END.len()]
            .copy_from_slice(&self.0);
        statement[Self::PRAGMA_REKEY_AND_ARGUMENT_LENGTH - Self::PRAGMA_END.len()..]
            .copy_from_slice(Self::PRAGMA_END);
        statement
    }

    fn to_hex_digit(n: u8) -> u8 {
        match n {
            0..=9 => b'0' + n,
            10..=15 => b'a' + (n - 10),
            _ => unreachable!(),
        }
    }
}
