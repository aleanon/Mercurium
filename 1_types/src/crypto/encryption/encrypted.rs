use std::{marker::PhantomData, num::NonZeroU32, str::Utf8Error};

use bip39::{ErrorKind, Mnemonic};
use ring::{aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, UnboundKey, AES_256_GCM, NONCE_LEN}, rand::{SecureRandom, SystemRandom}};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;
use crate::crypto::Salt;

use super::{crypto_error::CryptoError, encrypt::Encrypt, encryption_algorithm::EncryptionAlgorithm, key_salt_pair::KeySaltPair};

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("General Error")]
    General
}


struct EncryptedNonceSequence(Nonce);

impl EncryptedNonceSequence {
    pub fn new() -> Result<Self, EncryptionError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| EncryptionError::General)?;

        Ok(Self(Nonce::assume_unique_for_key(nonce_bytes)))
    }

    pub fn with_nonce(nonce: &Nonce) -> Self {
        Self(Nonce::assume_unique_for_key(nonce.as_ref().clone()))
    }

    pub fn get_current_as_bytes(&self) -> [u8; NONCE_LEN] {
        self.0.as_ref().clone()
    }
}


impl NonceSequence for EncryptedNonceSequence {
    fn advance(&mut self) -> Result<ring::aead::Nonce, ring::error::Unspecified> {
        let nonce = Nonce::assume_unique_for_key(self.get_current_as_bytes());
        Ok(nonce)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encrypted<T: Encrypt> {
    data: Vec<u8>,
    salt: Salt,
    nonce_bytes: [u8; 12],
    _marker: PhantomData<T>
}

impl<T> Encrypted<T> 
    where T: Encrypt
    {
    
    pub(crate) fn encrypt(key_salt_pair: KeySaltPair<T>, mut data: Vec<u8>) -> Result<Self, T::Error> {

        let nonce_sequence = EncryptedNonceSequence::new()
            .map_err(|_| CryptoError::FailedToCreateNonce)?;

        let nonce_bytes = nonce_sequence.get_current_as_bytes();

        let unbound_key = UnboundKey::new( T::ENCRYPTION_ALGORITHM.into(), key_salt_pair.key().as_bytes())
            .map_err(|_| CryptoError::WrongKeyLength { expected: T::KEY_LENGTH, actual: key_salt_pair.key().as_bytes().len() })?;

        let mut sealing_key = ring::aead::SealingKey::new(unbound_key, nonce_sequence);

        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut data)
            .map_err(|_| CryptoError::FailedToEncryptData)?;

        Ok(Self {
            data,
            salt: key_salt_pair.into_salt(),
            nonce_bytes,
            _marker: PhantomData,
        })
    }

    pub fn decrypt(&self, secret: &str) -> Result<T, T::Error> {
        let encryption_key= T::create_key(secret, &self.salt);
        let unbound_key = UnboundKey::new(&AES_256_GCM, encryption_key.as_bytes())
            .map_err(|_| CryptoError::WrongKeyLength { expected: T::KEY_LENGTH, actual: encryption_key.as_bytes().len() })?;

        let nonce_sequence = EncryptedNonceSequence::with_nonce(&Nonce::assume_unique_for_key(
            self.nonce_bytes.clone(),
        ));
        let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);
        
        let mut data = self.data.clone();

        let decrypted = opening_key
            .open_in_place(Aad::empty(), &mut data)
            .map_err(|_| CryptoError::FailedToDecryptData)?;

        let result:T = T::from_decrypted_data(decrypted)?;

        data.zeroize();

        Ok(result)
    }

    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    pub fn encrypted_data(&self) -> &[u8] {
        &self.data
    }
}




#[cfg(test)]
mod tests {
    use bip39::Language;

    use super::*;

    #[derive(Debug, Error)]
pub enum MyError {
    #[error("Failed to parse decrypted data")]
    InvalidPhraseForMnemonic(#[from] ErrorKind),
    #[error("Invalid utf-8 in decrypted data")]
    InvalidUtf8InDecryptedData(#[from] Utf8Error),
    #[error("Encrypt Error: {0}")]
    EncryptError(#[from] CryptoError)
}



impl Encrypt for Mnemonic {
    type Error = MyError;
    const ENCRYPTION_ALGORITHM: EncryptionAlgorithm = EncryptionAlgorithm::Aes_256_GCM;
    const KEY_ITERATIONS: NonZeroU32 = NonZeroU32::new(10000000).unwrap();

    fn data_to_encrypt(&self) -> Result<impl Into<Vec<u8>>, Self::Error> {
        Ok(self.phrase())
    }

    fn from_decrypted_data(data: &[u8]) -> Result<Self, Self::Error> {
        Ok(Mnemonic::from_phrase(std::str::from_utf8(data)?, bip39::Language::English)?)
    }

}

    #[test]
    fn test_encryption_decryption() {
        let phrase = "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis";
        let mnemonic = Mnemonic::from_phrase(
            phrase,
            Language::English
        ).expect("Failed to create mnemonic");

        let secret = "password123";
        let encrypted = mnemonic.encrypt(secret).expect("Failed to encrypt mnemonic");

        if let Ok(string) = std::str::from_utf8(encrypted.encrypted_data()) {
            assert_ne!(string, phrase)
        }

        let decrypted = encrypted.decrypt(secret).expect("Failed to decrypt mnemonic");
        assert_eq!(mnemonic.phrase(), decrypted.phrase());
    }
}