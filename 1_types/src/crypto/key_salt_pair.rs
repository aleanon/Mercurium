use deps::*;

use std::mem;

use zeroize::ZeroizeOnDrop;

use super::{CryptoError, Key, KeyType, Salt};

#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct KeySaltPair<T>
where
    T: KeyType,
{
    key: Key<T>,
    salt: Salt,
}

impl<T> KeySaltPair<T>
where
    T: KeyType,
{
    pub fn new(source: &str) -> Result<Self, CryptoError> {
        let salt = Salt::new()?;
        let key = Key::new(source, &salt);
        Ok(Self { key, salt })
    }

    pub fn from_salt(source: &str, salt: Salt) -> Self {
        Self {
            key: Key::new(source, &salt),
            salt,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU32;

    struct TestKeyType;
    impl KeyType for TestKeyType {
        const KEY_LENGTH: usize = 32;
        const ITERATIONS: NonZeroU32 = NonZeroU32::MIN;
    }

    #[test]
    fn from_salt_keeps_salt_and_derives_matching_key() {
        let salt = Salt::from([3u8; Salt::LENGTH]);
        let pair = KeySaltPair::<TestKeyType>::from_salt("pw", salt.clone());

        assert_eq!(pair.salt().as_bytes(), salt.as_bytes());
        let expected = Key::<TestKeyType>::new("pw", &salt);
        assert_eq!(pair.key().as_bytes(), expected.as_bytes());
    }

    #[test]
    fn into_inner_splits_key_and_salt() {
        let salt = Salt::from([4u8; Salt::LENGTH]);
        let pair = KeySaltPair::<TestKeyType>::from_salt("pw", salt.clone());
        let (key, out_salt) = pair.into_inner();
        assert_eq!(out_salt.as_bytes(), salt.as_bytes());
        assert_eq!(key.as_bytes(), Key::<TestKeyType>::new("pw", &salt).as_bytes());
    }

    #[test]
    fn into_salt_and_into_key_extract_each_half() {
        let salt = Salt::from([5u8; Salt::LENGTH]);
        let by_salt = KeySaltPair::<TestKeyType>::from_salt("pw", salt.clone()).into_salt();
        assert_eq!(by_salt.as_bytes(), salt.as_bytes());

        let by_key = KeySaltPair::<TestKeyType>::from_salt("pw", salt.clone()).into_key();
        assert_eq!(by_key.as_bytes(), Key::<TestKeyType>::new("pw", &salt).as_bytes());
    }
}
