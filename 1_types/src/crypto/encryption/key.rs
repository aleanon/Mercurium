use std::{fmt::Debug, num::NonZeroU32};

use ring::pbkdf2::{self, Algorithm, PBKDF2_HMAC_SHA256};
use zeroize::ZeroizeOnDrop;

use crate::crypto::Salt;

use super::{encrypt::Encrypt, encryption_algorithm::{encryption_key_length, EncryptionAlgorithm}};

pub trait KeyOptions {
    const ITERATIONS: NonZeroU32;
    const ALGORITHM: EncryptionAlgorithm;
    const KEY_LENGTH: usize = encryption_key_length(Self::ALGORITHM);

    fn create_key(secret: &str, salt: &Salt) -> Key {
        Key::new(Self::ITERATIONS, Self::ALGORITHM, secret, salt)
    }
}

impl<T> KeyOptions for T where T: Encrypt {
    const ITERATIONS: NonZeroU32 = T::KEY_ITERATIONS;
    const ALGORITHM: EncryptionAlgorithm = T::ENCRYPTION_ALGORITHM;
}

#[derive(ZeroizeOnDrop)]
pub enum Key {
    None,
    Key128Bit([u8;Self::KEY_128_LENGTH]),
    Key256Bit([u8;Self::KEY_256_LENGTH]),
}

impl Key {
    const KEY_128_LENGTH: usize = 16;
    const KEY_256_LENGTH: usize = 32;

    pub fn new(iterations: NonZeroU32, encryption_algo: EncryptionAlgorithm, secret: &str, salt: &Salt) -> Self {
        match encryption_algo {
            EncryptionAlgorithm::Aes_128_GCM => Key::Key128Bit([0u8; Self::KEY_128_LENGTH]).fill_key(iterations, secret, salt),
            EncryptionAlgorithm::Aes_256_GCM
            | EncryptionAlgorithm::ChaCha20_Poly1305 => Key::Key256Bit([0u8; Self::KEY_256_LENGTH]).fill_key(iterations, secret, salt),
        }
    }

    fn fill_key(mut self, iterations: NonZeroU32, secret: &str, salt: &Salt) -> Self {
        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            iterations,
            salt.as_bytes(),
            secret.as_bytes(),
            self.as_mut_bytes(),
        );
        self
    }

    fn as_mut_bytes(&mut self) -> &mut [u8] {
        match self {
            Key::Key128Bit(key) => key.as_mut(),
            Key::Key256Bit(key) => key.as_mut(),
            _ => &mut []
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Key::Key128Bit(key) => key.as_ref(),
            Key::Key256Bit(key) => key.as_ref(),
            _ => &[]
        }
    } 
}


impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::None => write!(f, "None"),
            Key::Key128Bit(_) => write!(f, "Key128Bit(*)"),
            Key::Key256Bit(_) => write!(f, "Key256Bit(*)"),
        }
    }
}

