use ring::rand::{SecureRandom, SystemRandom};
use zeroize::ZeroizeOnDrop;
use super::{key::KEY_LENGTH, encryption_error::EncryptionError};
use serde::{Serialize, Deserialize};

#[derive(Debug, ZeroizeOnDrop, Serialize, Deserialize)]
pub struct Salt([u8; KEY_LENGTH]);


impl Salt{
    pub fn new() -> Result<Self, EncryptionError> {
        let mut salt = [0u8; KEY_LENGTH];
        SystemRandom::new().fill(&mut salt).map_err(|_| EncryptionError::FailedToCreateRandomValue)?;
        Ok(Self(salt))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_inner(self) -> [u8; KEY_LENGTH] {
        self.0
    }
}