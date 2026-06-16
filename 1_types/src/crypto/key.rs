use deps::*;

use super::salt::Salt;
use ring::pbkdf2::{self, PBKDF2_HMAC_SHA256};
use std::{fmt::Debug, marker::PhantomData, num::NonZeroU32};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub trait KeyType {
    const KEY_LENGTH: usize;
    const ITERATIONS: NonZeroU32;
}

const KEY_LENGTH: usize = 32;

#[derive(Clone, ZeroizeOnDrop, Zeroize)]
pub struct Key<T: KeyType> {
    inner: [u8; KEY_LENGTH],
    _marker: std::marker::PhantomData<T>,
}

impl<T: KeyType> Key<T> {
    // pub const LENGTH: usize = 32;

    pub fn new(source: &str, salt: &Salt) -> Self {
        // Change to generic array length through the KeyType trait when the functionality is stable
        let mut key = [0u8; KEY_LENGTH];

        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            T::ITERATIONS,
            salt.as_bytes(),
            source.as_bytes(),
            &mut key,
        );

        Self {
            inner: key,
            _marker: PhantomData,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn into_inner(mut self) -> [u8; KEY_LENGTH] {
        std::mem::take(&mut self.inner)
    }
}

impl<T> Default for Key<T>
where
    T: KeyType,
{
    fn default() -> Self {
        Self {
            inner: [0; KEY_LENGTH],
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for Key<T>
where
    T: KeyType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Key<{:?}>", std::any::type_name::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Salt;

    struct TestKeyType;
    impl KeyType for TestKeyType {
        const KEY_LENGTH: usize = 32;
        const ITERATIONS: NonZeroU32 = NonZeroU32::MIN; // fast; production types use many more
    }

    #[test]
    fn derivation_is_deterministic() {
        let salt = Salt::from([1u8; Salt::LENGTH]);
        let a = Key::<TestKeyType>::new("password", &salt);
        let b = Key::<TestKeyType>::new("password", &salt);
        assert_eq!(a.as_bytes(), b.as_bytes());
        assert_eq!(a.as_bytes().len(), KEY_LENGTH);
    }

    #[test]
    fn different_source_or_salt_yields_different_key() {
        let salt = Salt::from([1u8; Salt::LENGTH]);
        let other_salt = Salt::from([2u8; Salt::LENGTH]);
        let base = Key::<TestKeyType>::new("password", &salt);

        assert_ne!(
            base.as_bytes(),
            Key::<TestKeyType>::new("different", &salt).as_bytes()
        );
        assert_ne!(
            base.as_bytes(),
            Key::<TestKeyType>::new("password", &other_salt).as_bytes()
        );
    }

    #[test]
    fn default_key_is_zeroed_and_into_inner_roundtrips() {
        assert!(Key::<TestKeyType>::default().as_bytes().iter().all(|&b| b == 0));
        let salt = Salt::from([9u8; Salt::LENGTH]);
        let key = Key::<TestKeyType>::new("x", &salt);
        let bytes = key.as_bytes().to_vec();
        assert_eq!(key.into_inner().to_vec(), bytes);
    }

    #[test]
    fn debug_does_not_leak_key_bytes() {
        let debug = format!("{:?}", Key::<TestKeyType>::default());
        assert!(debug.starts_with("Key<"));
        assert!(!debug.contains("0, 0, 0"));
    }
}
