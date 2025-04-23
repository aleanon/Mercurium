use deps::*;

use bip39::Mnemonic;
use store::{AppDataDb, DataBase};
use types::{
    crypto::{EncryptedMnemonic, HashedPassword, Key, KeySaltPair, Salt},
    Account, AppError, Network,
};

/// Encrypts the mnemonic and stores it using the OS credentials system.
/// It also makes the initial creation of the database and stores the passed in accounts
pub async fn create_new_wallet_with_accounts(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    mut db_key_salt: KeySaltPair<DataBase>,
    mnemonic_key_salt: KeySaltPair<EncryptedMnemonic>,
    password_hash: HashedPassword,
    accounts: &[Account],
    network: Network,
) -> Result<(), AppError> {
    let encrypted_mnemonic = EncryptedMnemonic::new_with_key_and_salt(
        mnemonic,
        seed_password.unwrap_or(""),
        mnemonic_key_salt,
    )
    .map_err(|err| AppError::Fatal(err.to_string()))?;

    crate::credentials::store_encrypted_mnemonic(&encrypted_mnemonic)
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    crate::credentials::store_db_encryption_salt(db_key_salt.take_salt())?;

    let db = AppDataDb::load(network, db_key_salt.take_key())
        .await
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    db.upsert_password_hash(password_hash).await
        .map_err(|err| AppError::Fatal(err.to_string()))?;
    db.upsert_accounts(accounts.to_vec()).await.ok();

    Ok(())
}
