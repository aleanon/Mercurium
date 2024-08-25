use bip39::Mnemonic;
use store::Db;
use types::{
    crypto::{EncryptedMnemonic, Password},
    Account, AppError, Network,
};

pub fn create_new_wallet_with_accounts(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    app_password: &Password,
    accounts: Vec<Account>,
    network: Network,
    db: &mut Db,
) -> Result<(), AppError> {
    let (_key, _salt) = app_password
        .derive_new_db_encryption_key()
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    let encrypted_mnemonic = EncryptedMnemonic::new(mnemonic, &app_password)
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    crate::credentials::store_encrypted_mnemonic(&encrypted_mnemonic)
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    db.upsert_accounts(&accounts).ok();

    Ok(())
}
