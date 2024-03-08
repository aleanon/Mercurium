use std::num::NonZeroU32;

use ring::pbkdf2::{PBKDF2_HMAC_SHA256, self};
use zeroize::ZeroizeOnDrop;
use super::{ salt::Salt,password::Password};


pub const KEY_LENGTH: usize = 32;

#[derive(Debug, ZeroizeOnDrop)]
pub struct Key([u8; KEY_LENGTH]);


impl Key{

    //Iteration counts needs to be > 0 else the program will crash
    const DB_KEY_ITERATIONS: u32 = 200000;
    const MNEMONIC_KEY_ITERATIONS: u32 = 2000000;

    pub fn db_encryption_key(salt: &Salt, password: &Password) -> Self{
        let mut key = [0u8; KEY_LENGTH];
        let iterations = NonZeroU32::new(Self::DB_KEY_ITERATIONS)
            .unwrap_or_else(|| unreachable!("{}:{} Attempted to create NonZeroU32 from a 0 value", module_path!(), line!()));

        pbkdf2::derive(PBKDF2_HMAC_SHA256, iterations, salt.as_bytes(), password.as_str().as_bytes(), &mut key);
        
        Self(key)
    }

    pub fn mnemonic_encryption_key(salt: &Salt, password: &Password) -> Self{
        let mut key = [0u8; KEY_LENGTH];
        let iterations = NonZeroU32::new(Self::MNEMONIC_KEY_ITERATIONS)
            .unwrap_or_else(|| unreachable!("{}:{} Attempted to create NonZeroU32 from a 0 value", module_path!(), line!()));

        pbkdf2::derive(PBKDF2_HMAC_SHA256, iterations, salt.as_bytes(), password.as_str().as_bytes(), &mut key);
        
        Self(key)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_ref(&self) -> &[u8; KEY_LENGTH] {
        &self.0
    }

    pub fn to_inner(self) -> [u8; KEY_LENGTH] {
        self.0
    }
}