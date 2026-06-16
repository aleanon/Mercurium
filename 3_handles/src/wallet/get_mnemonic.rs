use deps::*;

use bip39::Mnemonic;
use types::{AppError, crypto::Password};

/// Decrypts and returns the wallet mnemonic (and its optional seed password) using the user's
/// login password.
///
/// This is the signing-session primitive: rather than caching the decrypted mnemonic in memory
/// for the lifetime of the unlocked wallet, callers re-decrypt it for each signing operation and
/// drop it immediately afterwards. The mnemonic is zeroized on drop. Decryption derives a key
/// with a high iteration count, so callers should run this off the UI thread.
pub fn get_decrypted_mnemonic(password: &Password) -> Result<(Mnemonic, Password), AppError> {
    let encrypted = crate::credentials::get_encrypted_mnemonic()
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    encrypted
        .decrypt_mnemonic(password)
        .map_err(|err| AppError::NonFatal(types::Notification::Info(err.to_string())))
}
