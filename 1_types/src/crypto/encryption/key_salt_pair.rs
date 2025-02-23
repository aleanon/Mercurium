use std::marker::PhantomData;

use crate::crypto::Salt;

use super::{crypto_error::CryptoError, key::{Key, KeyOptions}};



pub struct KeySaltPair<T: KeyOptions> {
    key: Key,
    salt: Salt,
    _marker: PhantomData<T>
}

impl<T:KeyOptions> KeySaltPair<T> {
    
    pub fn new(source: &str) -> Result<Self, CryptoError> {
        let salt = Salt::new().map_err(|_|CryptoError::FailedToCreateSalt)?;
        let key = T::create_key(source, &salt);
        Ok(Self { key, salt, _marker: PhantomData })
    }

    pub fn from_salt(source: &str, salt: Salt) -> Self {
        Self {
            key: T::create_key(source, &salt),
            salt: salt,
            _marker: PhantomData,
        }
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    /// Takes the [Key] and [Salt], dropping the empty [KeyAndSalt] 
    pub fn into_inner(mut self) -> (Key, Salt) {
        (std::mem::replace(&mut self.key, Key::None), std::mem::take(&mut self.salt))
    }

    /// Takes the [Salt], dropping the [Key]
    pub fn into_salt(mut self) -> Salt {
        std::mem::take(&mut self.salt)
    }

    /// Takes the [Key], dropping the [Salt]
    pub fn into_key(mut self) -> Key {
        std::mem::replace(&mut self.key, Key::None)
    }

    /// Takes the [Salt], leaving the [Key]
    pub fn take_salt(&mut self) -> Salt {
        std::mem::take(&mut self.salt)
    }

    /// Takes the [Key], leaving the [Salt]
    pub fn take_key(&mut self) -> Key {
        std::mem::replace(&mut self.key, Key::None)
    }
}