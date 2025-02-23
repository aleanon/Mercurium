use std::num::NonZeroU32;

use ring::pbkdf2::{self, PBKDF2_HMAC_SHA256};

use crate::crypto::Salt;

use super::{encrypt::Encrypt, encryption_algorithm::EncryptionAlgorithm};



trait TKey: Sized + Default {

    fn create_key(algorithm: EncryptionAlgorithm, iterations: NonZeroU32, secret: &str, salt: &Salt) -> Self {
        let mut key = Self::default();
        key.fill_key(iterations, secret.as_bytes(), salt);
        key
    }    

    fn key_data(&mut self) -> &mut [u8];

    fn fill_key(mut self, iterations: NonZeroU32, secret: &[u8], salt: &Salt) -> Self {
        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            iterations,
            salt.as_bytes(),
            secret,
            &mut self.key_data(),
        );
        self
    }
}

impl<T> TKey for T where T: Encrypt {

} 

pub struct Key128Bit([u8;Self::LENGTH]);

impl Key128Bit {
    const LENGTH: usize = 16;
}

impl Default for Key128Bit {
    fn default() -> Self {
        Key128Bit([0u8; Self::LENGTH])
    }
}

impl TKey for Key128Bit {
    fn create_key(algorithm: EncryptionAlgorithm, iterations: NonZeroU32, secret: &str, salt: &Salt) -> Key {
        
    }

    fn key_data(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

pub struct Key256Bit([u8; Self::LENGTH]);

impl Key256Bit {
    const LENGTH: usize = 32;
}

impl Default for Key256Bit {
    fn default() -> Self {
        Key256Bit([0u8; Self::LENGTH])
    }
}

impl TKey for Key256Bit {
    fn key_data(&mut self) -> &mut [u8] {
        &mut self.0
    }
}