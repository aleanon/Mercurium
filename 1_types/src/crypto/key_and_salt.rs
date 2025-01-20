
use std::mem;

use zeroize::ZeroizeOnDrop;

use super::{CryptoError, Key, KeyType, Salt};

#[derive(Debug, ZeroizeOnDrop)]
pub struct KeyAndSalt<T> 
    where 
        T: KeyType,
{
    key: Key<T>,
    salt: Salt,
}

impl<T> KeyAndSalt<T> 
    where
        T: KeyType,
{
    pub fn new(source: &str) -> Result<Self, CryptoError> {
        let salt = Salt::new()?;
        let key = Key::new(source, &salt);
        Ok(Self { key, salt})
    }

    pub fn from_salt(source: &str, salt: Salt) -> Self {
        Self {
            key: Key::new(source, &salt),
            salt: salt,
        }
    }

    pub fn key(&self) -> &Key<T> {
        &self.key
    }

    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    /// Takes the [Key] and [Salt], dropping the empty [KeyAndSalt] 
    pub fn into_inner(mut self) -> (Key<T>, Salt) {
        (mem::take(&mut self.key), mem::take(&mut self.salt))
    }

    /// Takes the [Salt], dropping the [Key]
    pub fn into_salt(mut self) -> Salt {
        mem::take(&mut self.salt)
    }

    /// Takes the [Key], dropping the [Salt]
    pub fn into_key(mut self) -> Key<T> {
        mem::take(&mut self.key)
    }

    /// Takes the [Salt], leaving the [Key]
    pub fn take_salt(&mut self) -> Salt {
        mem::take(&mut self.salt)
    }

    /// Takes the [Key], leaving the [Salt]
    pub fn take_key(&mut self) -> Key<T> {
        mem::take(&mut self.key)
    }
}



