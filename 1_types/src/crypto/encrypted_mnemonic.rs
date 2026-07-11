use deps::*;

use core::str;
use std::num::NonZeroU32;

use super::sealed::{self, SealedBlob};
use super::{KeySaltPair, KeyType, Password, Salt};
use bip39::Mnemonic;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

/// Layout version of the [`EncryptedMnemonic`] struct itself (independent of the
/// AEAD [`SealedBlob`] version). Forward-looking hook; there is no stored data
/// to migrate today.
const ENCRYPTED_MNEMONIC_VERSION: u16 = 1;

#[derive(Error, Debug)]
pub enum EncryptedMnemonicError {
    #[error("Failed to create random value")]
    FailedToCreateRandomValue,
    #[error("Failed to encrypt mnemonic")]
    FailedToEncryptData,
    #[error("Failed to decrypt mnemonic")]
    FailedToDecryptData,
    #[error("Failed to parse, invalid utf-8")]
    InvalidUtf8,
    #[error("Unsupported EncryptedMnemonic version: {0}")]
    UnsupportedVersion(u16),
    #[error("Failed to construct mnemonic")]
    FailedToConstructMnemonic,
}

impl From<sealed::SealError> for EncryptedMnemonicError {
    fn from(err: sealed::SealError) -> Self {
        match err {
            sealed::SealError::Rng => Self::FailedToCreateRandomValue,
            sealed::SealError::Seal | sealed::SealError::KeyInit => Self::FailedToEncryptData,
            sealed::SealError::Open => Self::FailedToDecryptData,
            sealed::SealError::UnsupportedVersion(v) => Self::UnsupportedVersion(v),
        }
    }
}

/// The seed phrase and its optional seed password, each encrypted independently
/// under the same password-derived key. Each field is its own [`SealedBlob`]
/// with its **own** fresh nonce, so the two ciphertexts never share a
/// (key, nonce) pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMnemonic {
    format_version: u16,
    seed_phrase: SealedBlob,
    seed_password: SealedBlob,
    salt: Salt,
}

impl EncryptedMnemonic {
    /// Slow constructor: derives a fresh key+salt from `password` (high PBKDF2
    /// iteration count — run this off the UI thread).
    pub fn new(
        mnemonic: &Mnemonic,
        seed_password: &str,
        password: &Password,
    ) -> Result<Self, EncryptedMnemonicError> {
        let key_and_salt: KeySaltPair<EncryptedMnemonic> = KeySaltPair::new(password.as_str())
            .map_err(|_err| EncryptedMnemonicError::FailedToCreateRandomValue)?;
        Self::seal_with_key_and_salt(mnemonic, seed_password, key_and_salt)
    }

    /// Fast constructor with a pre-generated encryption key + salt.
    pub fn new_with_key_and_salt(
        mnemonic: &Mnemonic,
        seed_password: &str,
        encryption_key_salt: KeySaltPair<EncryptedMnemonic>,
    ) -> Result<Self, EncryptedMnemonicError> {
        Self::seal_with_key_and_salt(mnemonic, seed_password, encryption_key_salt)
    }

    fn seal_with_key_and_salt(
        mnemonic: &Mnemonic,
        seed_password: &str,
        key_and_salt: KeySaltPair<EncryptedMnemonic>,
    ) -> Result<Self, EncryptedMnemonicError> {
        let key = key_and_salt.key().as_bytes();

        // Two independent seals, each generating its own fresh nonce internally.
        let phrase_bytes: Vec<u8> = mnemonic.phrase().into();
        let seed_phrase = sealed::seal(key, phrase_bytes)?;

        let seed_password_bytes: Vec<u8> = seed_password.into();
        let seed_password = sealed::seal(key, seed_password_bytes)?;

        Ok(Self {
            format_version: ENCRYPTED_MNEMONIC_VERSION,
            seed_phrase,
            seed_password,
            salt: key_and_salt.into_salt(),
        })
    }

    pub fn decrypt_mnemonic(
        &self,
        password: &Password,
    ) -> Result<(Mnemonic, Password), EncryptedMnemonicError> {
        if self.format_version != ENCRYPTED_MNEMONIC_VERSION {
            return Err(EncryptedMnemonicError::UnsupportedVersion(
                self.format_version,
            ));
        }

        let key = KeySaltPair::<EncryptedMnemonic>::from_salt(password.as_str(), self.salt.clone());
        let key_bytes = key.key().as_bytes();

        let mut phrase_bytes = sealed::open(key_bytes, &self.seed_phrase)?;
        let mut phrase = String::from_utf8(phrase_bytes.clone())
            .map_err(|_| EncryptedMnemonicError::InvalidUtf8)?;
        let mnemonic = Mnemonic::from_phrase(&phrase, bip39::Language::English)
            .map_err(|_| EncryptedMnemonicError::FailedToConstructMnemonic)?;

        let mut seed_password_bytes = sealed::open(key_bytes, &self.seed_password)?;
        let seed_password = {
            let s = str::from_utf8(&seed_password_bytes)
                .map_err(|_| EncryptedMnemonicError::InvalidUtf8)?;
            Password::from(s)
        };

        // Wipe the plaintext copies before returning.
        phrase.zeroize();
        phrase_bytes.zeroize();
        seed_password_bytes.zeroize();

        Ok((mnemonic, seed_password))
    }
}

impl KeyType for EncryptedMnemonic {
    const KEY_LENGTH: usize = 32;
    const ITERATIONS: std::num::NonZeroU32 = NonZeroU32::new(2000000).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_encrypted_mnemonic() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let encrypted_mnemonic =
            EncryptedMnemonic::new(&mnemonic, "", &password).unwrap_or_else(|err| panic!("{err}"));

        let (decrypted, _) = encrypted_mnemonic.decrypt_mnemonic(&password).unwrap();
        assert_eq!(decrypted.phrase(), mnemonic.phrase());
    }

    #[test]
    fn test_decrypting_mnemonic() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let seed_password = "SomePasswordfor74s33dphrases";
        let encrypted_mnemonic = EncryptedMnemonic::new(&mnemonic, seed_password, &password)
            .unwrap_or_else(|err| panic!("{err}"));

        let (decrypted_mnemonic, decrypted_password) = encrypted_mnemonic
            .decrypt_mnemonic(&password)
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(mnemonic.phrase(), decrypted_mnemonic.phrase());
        assert_eq!(seed_password, decrypted_password.as_str());
    }

    #[test]
    fn seed_phrase_and_seed_password_use_distinct_nonces() {
        // Regression test for the GCM nonce-reuse bug: the two blobs must never
        // share a nonce, even when both plaintexts happen to be identical.
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let em = EncryptedMnemonic::new(&mnemonic, mnemonic.phrase(), &password).unwrap();
        let json = serde_json::to_value(&em).unwrap();
        assert_ne!(
            json["seed_phrase"]["nonce"], json["seed_password"]["nonce"],
            "the two seals must use distinct nonces"
        );
    }

    #[test]
    fn wrong_password_fails_to_decrypt() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let em =
            EncryptedMnemonic::new(&mnemonic, "", &Password::from("correct-horse")).unwrap();
        assert!(em.decrypt_mnemonic(&Password::from("wrong-horse")).is_err());
    }
}
