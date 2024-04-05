use bip39::Mnemonic;
use store::Db;
use types::{crypto::Password, AppError, Network};

use crate::EncryptedMnemonic;

const CREDENTIALS_STORE_NAME: &str = "Ravaultmnemonic";

pub fn create_new_wallet(mnemonic: &Mnemonic, password: &Password, account_name: String, network: Network, db: &mut Db) -> Result<(), AppError> {
  let (key, salt) = password.derive_new_db_encryption_key()
    .map_err(|err| AppError::NonFatal(Box::new(err)))?;

  let encrypted_mnemonic = EncryptedMnemonic::new(mnemonic, &password)
    .map_err(|err| AppError::Fatal(Box::new(err)))?;

  encrypted_mnemonic.save_to_store(CREDENTIALS_STORE_NAME)
    .map_err(|err| AppError::Fatal(Box::new(err)))?;

  let account = super::create_account::create_account_from_mnemonic(
      mnemonic, 0, account_name, network
  );

  db.update_account(&account).ok();

  Ok(())
} 