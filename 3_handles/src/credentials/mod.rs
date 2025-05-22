mod delete;
mod get_credentials;
mod store_credentials;
#[cfg(test)]
mod tests;

pub use delete::{delete_encrypted_mnemonic, delete_salt};
pub use get_credentials::{get_db_encryption_salt, get_encrypted_mnemonic};
pub use store_credentials::{store_db_encryption_salt, store_encrypted_mnemonic};

#[cfg(windows)]
pub(crate) const SALT_TARGET_NAME: &'static str = "l4h4c5aPo1ULu3dLQjCYrq2TJNY3wZiYwGL4jTOZ1Lk=";
#[cfg(windows)]
pub(crate) const ENCRYPTED_MNEMONIC_TARGET_NAME: &'static str =
    "Bk3oMH8tphurhYE3b/U/a4k03oefVrATNCFvWKz6FxA=";

#[cfg(unix)]
pub(crate) const SALT_TARGET_NAME: &'static str = "db_salt.json";
#[cfg(unix)]
pub(crate) const ENCRYPTED_MNEMONIC_TARGET_NAME: &'static str = "mnemonic.json";
