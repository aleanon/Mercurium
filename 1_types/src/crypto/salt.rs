use deps::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_salt_has_correct_length_and_is_random() {
        let a = Salt::new().unwrap();
        let b = Salt::new().unwrap();
        assert_eq!(a.as_bytes().len(), Salt::LENGTH);
        // Two random salts must (overwhelmingly likely) differ.
        assert_ne!(a.as_bytes(), b.as_bytes());
    }

    #[test]
    fn default_salt_is_zeroed() {
        assert!(Salt::default().as_bytes().iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_array_and_to_inner_roundtrip() {
        let bytes = [7u8; Salt::LENGTH];
        assert_eq!(Salt::from(bytes).to_inner(), bytes);
    }

    #[test]
    fn try_from_vec_validates_length() {
        assert!(Salt::try_from(vec![1u8; Salt::LENGTH]).is_ok());
        assert!(Salt::try_from(vec![1u8; 10]).is_err());
    }

    #[test]
    fn debug_redacts_contents() {
        assert_eq!(format!("{:?}", Salt::default()), "Salt(*)");
    }
}
