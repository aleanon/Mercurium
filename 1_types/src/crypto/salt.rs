use super::encryption_error::EncryptionError;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

#[derive(Debug, ZeroizeOnDrop, Serialize, Deserialize)]
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
