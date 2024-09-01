use bip39::Mnemonic;
use store::AsyncDb;
use types::{
    crypto::{DataBaseKey, EncryptedMnemonic, Key, Password, Salt},
    Account, AppError, Network,
};

pub async fn create_new_wallet_with_accounts(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    db_key: DataBaseKey,
    mnemonic_key_salt: (Key, Salt),
    accounts: &[Account],
    network: Network,
) -> Result<(), AppError> {
    let db = AsyncDb::new(network, db_key)
        .await
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    let encrypted_mnemonic = EncryptedMnemonic::new_with_key_and_salt(
        mnemonic,
        seed_password.unwrap_or(""),
        mnemonic_key_salt.0,
        mnemonic_key_salt.1,
    )
    .map_err(|err| AppError::Fatal(err.to_string()))?;

    crate::credentials::store_encrypted_mnemonic(&encrypted_mnemonic)
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    db.upsert_accounts(accounts).await.ok();

    Ok(())
}
