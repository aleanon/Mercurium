use super::encryption_error::CryptoError;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::{array::TryFromSliceError, fmt::Debug};
use zeroize::ZeroizeOnDrop;

#[cfg_attr(debug_assertions, derive(PartialEq, Eq))]
#[derive(Clone, ZeroizeOnDrop, Serialize, Deserialize)]
pub struct Salt([u8; Self::LENGTH]);

impl Salt {
    pub const LENGTH: usize = 32;

    pub fn new() -> Result<Self, CryptoError> {
        let mut salt = [0u8; Self::LENGTH];
        SystemRandom::new()
            .fill(&mut salt)
            .map_err(|_| CryptoError::FailedToCreateRandomValue)?;
        Ok(Self(salt))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_inner(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl Default for Salt {
    fn default() -> Self {
        Self([0u8; Self::LENGTH])
    }
}

impl From<[u8; Salt::LENGTH]> for Salt {
    fn from(value: [u8; Salt::LENGTH]) -> Self {
        Self(value)
    }
}

impl TryFrom<Vec<u8>> for Salt {
    type Error = TryFromSliceError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(value.as_slice().try_into()?))
    }
}

impl Debug for Salt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Salt(*)")
    }
}