use deps_two::*;

use super::{salt::Salt, KeySaltPair};
use ring::{aead::NonceSequence, pbkdf2::{self, PBKDF2_HMAC_SHA256}};
use serde::{Deserialize, Serialize};
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
        let mut key = [0u8;KEY_LENGTH];

        pbkdf2::derive(
            PBKDF2_HMAC_SHA256,
            T::ITERATIONS,
            salt.as_bytes(),
            source.as_bytes(),
            &mut key,
        );

        Self{inner: key, _marker: PhantomData}
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    pub fn into_inner(mut self) -> [u8;KEY_LENGTH] {
        std::mem::take(&mut self.inner)
    }
}

impl<T> Default for Key<T> 
    where 
        T: KeyType,
{
    fn default() -> Self {
        Self {
            inner: [0;KEY_LENGTH],
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

