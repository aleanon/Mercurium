use core::str;

use super::{Key, Password, Salt};
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
    FailedToEncryptData,
    #[error("Failed to decrypt mnemonic")]
    FailedToDecryptData,
    #[error("Failed to parse, invalid utf-8")]
    InvalidUtf8,
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
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMnemonic {
    encrypted_seed_phrase: Vec<u8>,
    encrypted_seed_password: Vec<u8>,
    salt: Salt,
    nonce_bytes: [u8; NONCE_LEN],
}

impl EncryptedMnemonic {
    pub fn new(
        mnemonic: &Mnemonic,
        seed_password: &str,
        password: &Password,
    ) -> Result<Self, EncryptedMnemonicError> {
        let mut mnemonic_encrypted: Vec<u8> = mnemonic.phrase().into();
        let mut seed_password_encrypted: Vec<u8> = seed_password.into();
        let (mut key, salt) = password
            .derive_new_mnemonic_encryption_key()
            .map_err(|_err| EncryptedMnemonicError::FailedToCreateRandomValue)?;

        let nonce_sequence = MnemonicNonceSequence::new()?;
        let nonce = nonce_sequence.get_current_as_bytes();

        let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_bytes())
            .map_err(|_| EncryptedMnemonicError::FailedToCreateUnboundKey)?;
        let mut sealing_key = ring::aead::SealingKey::new(unbound_key, nonce_sequence);

        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut mnemonic_encrypted)
            .map_err(|_| EncryptedMnemonicError::FailedToEncryptData)?;

        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut seed_password_encrypted)
            .map_err(|_| EncryptedMnemonicError::FailedToEncryptData)?;

        //TODO: Investigate zeroizing of the sealing key.
        key.zeroize();

        Ok(Self {
            encrypted_seed_phrase: mnemonic_encrypted,
            encrypted_seed_password: seed_password_encrypted,
            salt: salt,
            nonce_bytes: nonce,
        })
    }

    pub fn new_with_key_and_salt(
        mnemonic: &Mnemonic,
        seed_password: &str,
        mut encryption_key: Key,
        salt: Salt,
    ) -> Result<Self, EncryptedMnemonicError> {
        let mut mnemonic_encrypted: Vec<u8> = mnemonic.phrase().into();
        let mut seed_password_encrypted: Vec<u8> = seed_password.into();

        let nonce_sequence = MnemonicNonceSequence::new()?;
        let nonce = nonce_sequence.get_current_as_bytes();

        let unbound_key = UnboundKey::new(&AES_256_GCM, encryption_key.as_bytes())
            .map_err(|_| EncryptedMnemonicError::FailedToCreateUnboundKey)?;
        let mut sealing_key = ring::aead::SealingKey::new(unbound_key, nonce_sequence);

        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut mnemonic_encrypted)
            .map_err(|_| EncryptedMnemonicError::FailedToEncryptData)?;

        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut seed_password_encrypted)
            .map_err(|_| EncryptedMnemonicError::FailedToEncryptData)?;

        //TODO: See if there is a way to zeroize the sealing key
        encryption_key.zeroize();

        Ok(Self {
            encrypted_seed_phrase: mnemonic_encrypted,
            encrypted_seed_password: seed_password_encrypted,
            salt: salt,
            nonce_bytes: nonce,
        })
    }

    pub fn decrypt_mnemonic(
        &self,
        password: &Password,
    ) -> Result<(Mnemonic, Password), EncryptedMnemonicError> {
        let encryption_key = password.derive_mnemonic_encryption_key_from_salt(&self.salt);
        let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key.as_bytes())
            .map_err(|_| EncryptedMnemonicError::FailedToCreateUnboundKey)?;
        let nonce_sequence = MnemonicNonceSequence::with_nonce(&Nonce::assume_unique_for_key(
            self.nonce_bytes.clone(),
        ));
        let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

        let mut mnemonic_encrypted = self.encrypted_seed_phrase.clone();

        let phrase = opening_key
            .open_in_place(Aad::empty(), &mut mnemonic_encrypted)
            .map_err(|_| EncryptedMnemonicError::FailedToDecryptData)?;

        let mut phrase =
            String::from_utf8(phrase.to_vec()).map_err(|_| EncryptedMnemonicError::InvalidUtf8)?;

        let mnemonic = Mnemonic::from_phrase(&phrase, bip39::Language::English)
            .map_err(|_| EncryptedMnemonicError::FailedToConstructMnemonic)?;

        let mut seed_password_encrypted = self.encrypted_seed_password.clone();

        let seed_password_slice = opening_key
            .open_in_place(Aad::empty(), &mut seed_password_encrypted)
            .map_err(|_| EncryptedMnemonicError::FailedToDecryptData)?;

        let seed_pw_as_str = std::str::from_utf8(&seed_password_slice)
            .map_err(|_| EncryptedMnemonicError::InvalidUtf8)?;

        let seed_password = Password::from(seed_pw_as_str);

        //Zeroize the plaintext mnemonic and encryption key before dropping it
        //Todo: Investigate zeroizing of the opening key
        mnemonic_encrypted.zeroize();
        seed_password_encrypted.zeroize();
        phrase.zeroize();

        Ok((mnemonic, seed_password))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl EncryptedMnemonic {
        pub fn get_cypher_text(&self) -> Vec<u8> {
            self.encrypted_seed_phrase.clone()
        }
    }

    #[test]
    fn test_create_encrypted_mnemonic() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let encrypted_mnemonic = EncryptedMnemonic::new(&mnemonic.clone(), "", &password)
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
        let seed_password = "SomePasswordfor74s33dphrases";
        let encrypted_mnemonic =
            EncryptedMnemonic::new(&mnemonic.clone(), &seed_password, &password)
                .unwrap_or_else(|err| panic!("{err}"));

        println!(
            "{} \n{:?}",
            mnemonic.phrase(),
            encrypted_mnemonic.encrypted_seed_phrase
        );

        let (decrypted_mnemonic, decrypted_password) = encrypted_mnemonic
            .decrypt_mnemonic(&password)
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(mnemonic.phrase(), decrypted_mnemonic.phrase());
        assert_eq!(seed_password, decrypted_password.as_str());
    }
}
