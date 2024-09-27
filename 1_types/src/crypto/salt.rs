use super::encryption_error::EncryptionError;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::array::TryFromSliceError;
use zeroize::ZeroizeOnDrop;

#[cfg_attr(debug_assertions, derive(PartialEq, Eq))]
#[derive(Debug, Clone, ZeroizeOnDrop, Serialize, Deserialize)]
pub struct Salt([u8; Self::LENGTH]);

impl Salt {
    pub const LENGTH: usize = 32;

    pub fn new() -> Result<Self, EncryptionError> {
        let mut salt = [0u8; Self::LENGTH];
        SystemRandom::new()
            .fill(&mut salt)
            .map_err(|_| EncryptionError::FailedToCreateRandomValue)?;
        Ok(Self(salt))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_inner(self) -> [u8; Self::LENGTH] {
        self.0
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
