//! Encrypted Profile backup (export/import) — the seed-free wallet backup.
//!
//! The [`Profile`] (gateways, preferences, authorized dApps, factor-source metadata) is
//! serialized to JSON and encrypted with a key derived from the user's backup password
//! (PBKDF2) using AES-256-GCM. The encrypted blob layout is `salt(32) || nonce(12) ||
//! ciphertext+tag`, so it is self-describing for decryption. This is the file the user can keep
//! in cloud storage; it contains **no seed phrase**.

use deps::*;

use std::num::NonZeroU32;
use std::path::Path;

use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
use ring::rand::{SecureRandom, SystemRandom};
use types::Profile;
use types::crypto::{Key, KeyType, Salt};

const NONCE_LEN: usize = 12;

/// Key-derivation parameters for profile backups.
struct ProfileBackupKey;
impl KeyType for ProfileBackupKey {
    const KEY_LENGTH: usize = 32;
    const ITERATIONS: NonZeroU32 = NonZeroU32::new(200_000).unwrap();
}

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("Failed to serialize profile: {0}")]
    Serialize(String),
    #[error("Failed to deserialize profile: {0}")]
    Deserialize(String),
    #[error("Encryption failed")]
    Encrypt,
    #[error("Decryption failed (wrong password or corrupt backup)")]
    Decrypt,
    #[error("Backup blob is malformed")]
    Malformed,
    #[error("Random generation failed")]
    Random,
    #[error("Backup file I/O failed: {0}")]
    Io(String),
}

/// Serializes and encrypts the profile with the backup `password`.
pub fn export_encrypted(profile: &Profile, password: &str) -> Result<Vec<u8>, BackupError> {
    let mut plaintext = serde_json::to_vec(profile).map_err(|e| BackupError::Serialize(e.to_string()))?;

    let salt = Salt::new().map_err(|_| BackupError::Random)?;
    let key = Key::<ProfileBackupKey>::new(password, &salt);

    let unbound = UnboundKey::new(&AES_256_GCM, key.as_bytes()).map_err(|_| BackupError::Encrypt)?;
    let sealing = LessSafeKey::new(unbound);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|_| BackupError::Random)?;

    sealing
        .seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::empty(),
            &mut plaintext,
        )
        .map_err(|_| BackupError::Encrypt)?;

    let mut blob = Vec::with_capacity(Salt::LENGTH + NONCE_LEN + plaintext.len());
    blob.extend_from_slice(salt.as_bytes());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&plaintext);
    Ok(blob)
}

/// Decrypts and deserializes a profile backup blob with the backup `password`.
pub fn import_encrypted(blob: &[u8], password: &str) -> Result<Profile, BackupError> {
    if blob.len() < Salt::LENGTH + NONCE_LEN {
        return Err(BackupError::Malformed);
    }
    let (salt_bytes, rest) = blob.split_at(Salt::LENGTH);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

    let salt = Salt::from(<[u8; Salt::LENGTH]>::try_from(salt_bytes).map_err(|_| BackupError::Malformed)?);
    let key = Key::<ProfileBackupKey>::new(password, &salt);

    let unbound = UnboundKey::new(&AES_256_GCM, key.as_bytes()).map_err(|_| BackupError::Decrypt)?;
    let opening = LessSafeKey::new(unbound);

    let nonce_array: [u8; NONCE_LEN] = nonce_bytes.try_into().map_err(|_| BackupError::Malformed)?;
    let mut in_out = ciphertext.to_vec();
    let plaintext = opening
        .open_in_place(Nonce::assume_unique_for_key(nonce_array), Aad::empty(), &mut in_out)
        .map_err(|_| BackupError::Decrypt)?;

    serde_json::from_slice(plaintext).map_err(|e| BackupError::Deserialize(e.to_string()))
}

/// Writes an encrypted profile backup to `path`.
pub fn save_to_file(
    profile: &Profile,
    password: &str,
    path: impl AsRef<Path>,
) -> Result<(), BackupError> {
    let blob = export_encrypted(profile, password)?;
    std::fs::write(path, blob).map_err(|e| BackupError::Io(e.to_string()))
}

/// Reads and decrypts a profile backup from `path`.
pub fn load_from_file(path: impl AsRef<Path>, password: &str) -> Result<Profile, BackupError> {
    let blob = std::fs::read(path).map_err(|e| BackupError::Io(e.to_string()))?;
    import_encrypted(&blob, password)
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{AuthorizedDapp, GatewayConfig, Network};

    fn sample_profile() -> Profile {
        let mut profile = Profile::new();
        profile.gateways.switch_to(GatewayConfig::stokenet());
        profile.upsert_authorized_dapp(AuthorizedDapp {
            dapp_definition_address: "account_rdx_dapp".to_string(),
            display_name: Some("Test".to_string()),
            origin: "https://x".to_string(),
            authorized_personas: vec![],
        });
        profile
    }

    #[test]
    fn export_then_import_roundtrips() {
        let profile = sample_profile();
        let blob = export_encrypted(&profile, "backup-password").unwrap();
        let restored = import_encrypted(&blob, "backup-password").unwrap();
        assert_eq!(profile, restored);
        assert_eq!(restored.gateways.current.network, Network::Stokenet);
    }

    #[test]
    fn wrong_password_fails_to_decrypt() {
        let blob = export_encrypted(&sample_profile(), "correct").unwrap();
        assert!(matches!(
            import_encrypted(&blob, "wrong"),
            Err(BackupError::Decrypt)
        ));
    }

    #[test]
    fn malformed_blob_is_rejected() {
        assert!(matches!(
            import_encrypted(&[0u8; 8], "pw"),
            Err(BackupError::Malformed)
        ));
    }

    #[test]
    fn save_and_load_file_roundtrips() {
        let profile = sample_profile();
        let mut path = std::env::temp_dir();
        path.push(format!("mercurium_profile_test_{}.bin", std::process::id()));
        save_to_file(&profile, "pw", &path).unwrap();
        let restored = load_from_file(&path, "pw").unwrap();
        assert_eq!(profile, restored);
        std::fs::remove_file(&path).ok();
    }
}
