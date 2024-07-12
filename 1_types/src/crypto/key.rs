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
        let iterations = NonZeroU32::new(Self::DB_KEY_ITERATIONS).unwrap_or_else(|| {
            unreachable!(
                "{}:{} Attempted to create NonZeroU32 from a 0 value",
                module_path!(),
                line!()
            )
        });

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
        let iterations = NonZeroU32::new(Self::MNEMONIC_KEY_ITERATIONS).unwrap_or_else(|| {
            unreachable!(
                "{}:{} Attempted to create NonZeroU32 from a 0 value",
                module_path!(),
                line!()
            )
        });

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

    pub fn into_hex_key(self) -> HexKey {
        HexKey::from_key(self)
    }

    pub fn to_inner(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

#[derive(Debug, ZeroizeOnDrop)]
pub struct HexKey([u8; Self::LENGTH]);

impl HexKey {
    const LENGTH: usize = Key::LENGTH * 2;

    pub fn from_key(key: Key) -> Self {
        let mut hex_key = [0u8; Self::LENGTH];
        for (i, byte) in key.as_bytes().iter().enumerate() {
            hex_key[i * 2] = Self::to_hex_digit(byte >> 4);
            hex_key[i * 2 + 1] = Self::to_hex_digit(byte & 0x0F);
        }
        Self(hex_key)
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_unreachable(debug_info!("Key contained non utf8 bytes"))
    }

    fn to_hex_digit(n: u8) -> u8 {
        match n {
            0..=9 => b'0' + n,
            10..=15 => b'a' + (n - 10),
            _ => unreachable!(),
        }
    }
}
