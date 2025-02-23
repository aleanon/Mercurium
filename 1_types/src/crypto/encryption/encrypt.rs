
use std::num::NonZeroU32;


use super::{crypto_error::CryptoError, encrypted::{Encrypted, EncryptionError}, encryption_algorithm::EncryptionAlgorithm, key::KeyOptions, key_salt_pair::KeySaltPair};

/// 
pub trait Encrypt: Sized + KeyOptions {
    type Error: From<CryptoError>;

    /// Number of hashing rounds when creating a key from the provided secret
    const KEY_ITERATIONS: NonZeroU32;
    const ENCRYPTION_ALGORITHM: EncryptionAlgorithm;    

    fn data_to_encrypt(&self) -> Result<impl Into<Vec<u8>>, Self::Error>;

    fn from_decrypted_data(data: &[u8]) -> Result<Self, Self::Error>;

    /// Encrypts data supplied from this type and wraps it in an [Encrypted<T>]
    fn encrypt(&self, secret: &str) -> Result<Encrypted<Self>, Self::Error> {
        Encrypted::encrypt(KeySaltPair::new(secret)?, self.data_to_encrypt()?.into())
    }
} 