use super::{password::Password, salt::Salt};
use ring::pbkdf2::{self, PBKDF2_HMAC_SHA256};
use std::num::NonZeroU32;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, ZeroizeOnDrop, Zeroize)]
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

    pub fn to_inner(self) -> [u8; Self::LENGTH] {
        self.0
    }
}
