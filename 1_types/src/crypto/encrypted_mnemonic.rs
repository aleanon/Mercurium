use std::borrow::BorrowMut;

use super::{Password, Salt};
use bip39::Mnemonic;
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, UnboundKey, AES_256_GCM, NONCE_LEN,
};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

#[derive(Error, Debug)]
pub enum EncryptedMnemonicError {
    #[error("Failed to create random value")]
    FailedToCreateRandomValue,
    #[error("Failed to create unbound key")]
    FailedToCreateUnboundKey,
    #[error("Failed to encrypt mnemonic")]
    FailedToEncryptMnemonic,
    #[error("Failed to decrypt mnemonic")]
    FailedToDecryptMnemonic,
    #[error("Failed to parse, invalid utf-8")]
    FailedToParseInvalidUtf8,
    #[error("Failed to parse EncryptedMnemonic")]
    FailedToParseEncryptedMnemonic,
    #[error("Failed to save mnemonic, {0}")]
    FailedToSaveCredentials(String),
    #[error("Failed to retrieve Credentials: {0}")]
    FailedToRetrieveCredentials(String),
    #[error("Failed to delete Credentials: {0}")]
    FailedToDeleteCredentials(String),
    #[error("Failed to construct mnemonic")]
    FailedToConstructMnemonic,
}

struct MnemonicNonceSequence(Nonce);

impl MnemonicNonceSequence {
    pub fn new() -> Result<Self, EncryptedMnemonicError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        SystemRandom::new()
            .fill(&mut nonce_bytes)
            .map_err(|_| EncryptedMnemonicError::FailedToCreateRandomValue)?;

        Ok(Self(Nonce::assume_unique_for_key(nonce_bytes)))
    }

    pub fn with_nonce(nonce: &Nonce) -> Self {
        Self(Nonce::assume_unique_for_key(nonce.as_ref().clone()))
    }

    pub fn get_current_as_bytes(&self) -> [u8; NONCE_LEN] {
        self.0.as_ref().clone()
    }
}

impl NonceSequence for MnemonicNonceSequence {
    fn advance(&mut self) -> Result<ring::aead::Nonce, ring::error::Unspecified> {
        let nonce = Nonce::assume_unique_for_key(self.get_current_as_bytes());
        Ok(nonce)

        // let nonce = Nonce::assume_unique_for_key(self.get_current());
        // let mut new_nonce = [0u8; NONCE_LEN];
        // SystemRandom::new().fill(&mut new_nonce)?;
        // self.0 = Nonce::assume_unique_for_key(new_nonce);
        // Ok(nonce)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMnemonic {
    cipher_text: Vec<u8>,
    salt: Salt,
    nonce_bytes: [u8; NONCE_LEN],
}

impl EncryptedMnemonic {
    pub fn new(mnemonic: &Mnemonic, password: &Password) -> Result<Self, EncryptedMnemonicError> {
        let mut mnemonic: Vec<u8> = mnemonic.phrase().into();
        let (mut key, salt) = password
            .derive_new_mnemonic_encryption_key()
            .map_err(|_err| EncryptedMnemonicError::FailedToCreateRandomValue)?;

        let nonce_sequence = MnemonicNonceSequence::new()?;
        let nonce = nonce_sequence.get_current_as_bytes();

        let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_bytes())
            .map_err(|_| EncryptedMnemonicError::FailedToCreateUnboundKey)?;
        let mut sealing_key = ring::aead::SealingKey::new(unbound_key, nonce_sequence);

        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut mnemonic)
            .map_err(|_| EncryptedMnemonicError::FailedToEncryptMnemonic)?;

        //TODO: Investigate if the unbound key and sealing key gets overwritten when going out of scope.
        key.zeroize();

        Ok(Self {
            cipher_text: mnemonic,
            salt: salt,
            nonce_bytes: nonce,
        })
    }

    pub fn decrypt_mnemonic(
        &self,
        password: &Password,
    ) -> Result<Mnemonic, EncryptedMnemonicError> {
        let encryption_key = password.derive_mnemonic_encryption_key_from_salt(&self.salt);
        let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key.as_bytes())
            .map_err(|_| EncryptedMnemonicError::FailedToCreateUnboundKey)?;
        let nonce_sequence = MnemonicNonceSequence::with_nonce(&Nonce::assume_unique_for_key(
            self.nonce_bytes.clone(),
        ));
        let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

        let mut data = self.cipher_text.clone();

        let phrase = opening_key
            .open_in_place(Aad::empty(), &mut data)
            .map_err(|_| EncryptedMnemonicError::FailedToDecryptMnemonic)?;

        let mut phrase = String::from_utf8(phrase.to_vec())
            .map_err(|_| EncryptedMnemonicError::FailedToParseInvalidUtf8)?;

        //The validity check will have to be passed when the mnemonic first was passed to create the encrypted mnemonic,
        //so the decrypted phrase will always be a valid mnemonic.
        let mnemonic = Mnemonic::from_phrase(&phrase, bip39::Language::English)
            .map_err(|_| EncryptedMnemonicError::FailedToConstructMnemonic)?;

        //Zeroize the plaintext mnemonic and encryption key before dropping it
        //Todo: Investigate zeroizing of the unbound and opening keys
        data.zeroize();
        phrase.zeroize();

        Ok(mnemonic)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl EncryptedMnemonic {
        pub fn get_cypher_text(&self) -> Vec<u8> {
            self.cipher_text.clone()
        }
    }

    #[test]
    fn test_create_encrypted_mnemonic() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let encrypted_mnemonic = EncryptedMnemonic::new(&mnemonic.clone(), &password)
            .unwrap_or_else(|err| panic!("{err}"));

        println!("{:?}\n{:?}", mnemonic, encrypted_mnemonic.get_cypher_text());

        assert_ne!(
            mnemonic.clone().into_phrase().as_bytes().to_vec(),
            encrypted_mnemonic.get_cypher_text()[0..mnemonic.phrase().len()]
        );
    }

    #[test]
    fn test_decrypting_mnemonic() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let encrypted_mnemonic = EncryptedMnemonic::new(&mnemonic.clone(), &password)
            .unwrap_or_else(|err| panic!("{err}"));

        println!(
            "{} \n{:?}",
            mnemonic.phrase(),
            encrypted_mnemonic.cipher_text
        );

        let decrypted = encrypted_mnemonic
            .decrypt_mnemonic(&password)
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(mnemonic.phrase(), decrypted.phrase())
    }
}
