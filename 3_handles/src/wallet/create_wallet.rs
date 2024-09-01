use bip39::Mnemonic;
use store::AsyncDb;
use types::{
    crypto::{EncryptedMnemonic, Password},
    Account, AppError, Network,
};

pub async fn create_new_wallet_with_accounts(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    app_password: &Password,
    accounts: &[Account],
    network: Network,
) -> Result<(), AppError> {
    let (db_key, _salt) = app_password
        .derive_new_db_encryption_key()
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    let db = AsyncDb::new(network, db_key)
        .await
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    let encrypted_mnemonic =
        EncryptedMnemonic::new(mnemonic, seed_password.unwrap_or(""), &app_password)
            .map_err(|err| AppError::Fatal(err.to_string()))?;

    crate::credentials::store_encrypted_mnemonic(&encrypted_mnemonic)
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    db.upsert_accounts(accounts).await.ok();

    Ok(())
}
