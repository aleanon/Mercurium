//! Misuse-resistant authenticated encryption (AES-256-GCM).
//!
//! This module is the *only* place that touches `ring::aead`. Its public API
//! makes nonce reuse inexpressible: [`seal`] generates a fresh random nonce on
//! every call and returns it bundled inside the [`SealedBlob`]; there is no way
//! for a caller to supply, reuse, or observe a raw nonce, and no `NonceSequence`
//! is exposed. This is the structural fix for the previous encrypted-mnemonic
//! bug, where one nonce was reused across two seals under the same key.

use deps::*;

use ring::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Version of the on-disk sealed-blob layout. Bumped only if the AEAD scheme or
/// blob encoding changes; [`open`] rejects any version it does not understand.
/// There is no stored data to migrate today — this is a forward-looking hook.
const SEALED_BLOB_VERSION: u16 = 1;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SealError {
    #[error("failed to generate a random nonce")]
    Rng,
    #[error("failed to initialise the AEAD key (wrong key length?)")]
    KeyInit,
    #[error("failed to encrypt")]
    Seal,
    #[error("failed to decrypt or authenticate")]
    Open,
    #[error("unsupported sealed-blob version: {0}")]
    UnsupportedVersion(u16),
}

/// A self-describing authenticated-encryption blob: a version tag, the fresh
/// per-blob nonce, and the ciphertext with its appended GCM tag. Non-exhaustive
/// so fields can be added without breaking matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SealedBlob {
    format_version: u16,
    nonce: [u8; NONCE_LEN],
    ciphertext: Vec<u8>,
}

/// Encrypt `plaintext` under `key` (a 32-byte AES-256 key). A fresh random nonce
/// is generated internally, so two calls with the same key are always safe.
///
/// `plaintext` is consumed (cleared) so the caller does not keep a second copy
/// of the secret around; the bytes move into the returned blob's ciphertext.
pub fn seal(key: &[u8], mut plaintext: Vec<u8>) -> Result<SealedBlob, SealError> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|_| SealError::Rng)?;

    let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, key).map_err(|_| SealError::KeyInit)?);
    // Safe use of `assume_unique_for_key`: nonce_bytes was just filled with
    // fresh CSPRNG output and is used for exactly this one seal.
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut plaintext)
        .map_err(|_| SealError::Seal)?;

    Ok(SealedBlob {
        format_version: SEALED_BLOB_VERSION,
        nonce: nonce_bytes,
        ciphertext: plaintext,
    })
}

/// Decrypt a [`SealedBlob`] produced by [`seal`] under the same `key`.
pub fn open(key: &[u8], blob: &SealedBlob) -> Result<Vec<u8>, SealError> {
    if blob.format_version != SEALED_BLOB_VERSION {
        return Err(SealError::UnsupportedVersion(blob.format_version));
    }

    let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, key).map_err(|_| SealError::KeyInit)?);
    let nonce = Nonce::assume_unique_for_key(blob.nonce);

    let mut in_out = blob.ciphertext.clone();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| SealError::Open)?;
    Ok(plaintext.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [7u8; 32];

    #[test]
    fn round_trips() {
        let blob = seal(&KEY, b"secret payload".to_vec()).unwrap();
        assert_eq!(open(&KEY, &blob).unwrap(), b"secret payload");
    }

    #[test]
    fn two_seals_use_distinct_nonces() {
        let a = seal(&KEY, b"same plaintext".to_vec()).unwrap();
        let b = seal(&KEY, b"same plaintext".to_vec()).unwrap();
        // The whole point of the fix: identical (key, plaintext) never reuses a
        // nonce, so the ciphertexts differ too.
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.ciphertext, b.ciphertext);
    }

    #[test]
    fn tampered_ciphertext_fails_to_open() {
        let mut blob = seal(&KEY, b"secret".to_vec()).unwrap();
        blob.ciphertext[0] ^= 0xff;
        assert_eq!(open(&KEY, &blob), Err(SealError::Open));
    }

    #[test]
    fn wrong_key_fails_to_open() {
        let blob = seal(&KEY, b"secret".to_vec()).unwrap();
        assert_eq!(open(&[9u8; 32], &blob), Err(SealError::Open));
    }

    #[test]
    fn unknown_version_is_rejected() {
        let mut blob = seal(&KEY, b"secret".to_vec()).unwrap();
        blob.format_version = 999;
        assert_eq!(open(&KEY, &blob), Err(SealError::UnsupportedVersion(999)));
    }

    #[test]
    fn wrong_key_length_is_key_init_error() {
        assert_eq!(seal(&[0u8; 16], b"x".to_vec()), Err(SealError::KeyInit));
    }
}
