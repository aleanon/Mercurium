use store::DataBase;
use types::crypto::{EncryptedMnemonic, KeySaltPair, Password};

use crate::wallet::initial::SetupError;

pub struct WalletEncryptionKeys {
    pub mnemonic_key_salt: KeySaltPair<EncryptedMnemonic>,
    pub db_key_salt: KeySaltPair<DataBase>,
}

impl WalletEncryptionKeys {
    pub fn new(password: &Password) -> Result<Self, SetupError> {
        let db_key_salt: KeySaltPair<DataBase> = KeySaltPair::new(password.as_str())?;
        let mnemonic_key_salt: KeySaltPair<EncryptedMnemonic> = KeySaltPair::new(password.as_str())?;
        Ok(Self { db_key_salt, mnemonic_key_salt})
    }

    pub fn from_keys_and_salt(mnemonic_key_salt: KeySaltPair<EncryptedMnemonic>, db_key_salt: KeySaltPair<DataBase>) -> Self {
        Self {
            mnemonic_key_salt,
            db_key_salt,
        }
    }
}