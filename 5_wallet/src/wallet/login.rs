use deps::*;

use std::sync::Arc;

use debug_print::debug_println;
use futures::TryFutureExt;
use data_stores::{AppDataDb, IconsDb};
use secrets_store::SecretsStore;
use types::{
    AppError, Network,
    crypto::{Key, Password},
};

/// Verifies the login `password` against the stored hash and, on success, loads the encrypted
/// databases for the session. Moved here from `handles::wallet::login`.
pub async fn perform_login_check(
    secrets: &Arc<dyn SecretsStore>,
    network: Network,
    password: &Password,
) -> Result<(), AppError> {
    let salt = secrets.get_db_encryption_salt()?;
    let password_hash = password.derive_db_encryption_key_hash_from_salt(&salt);

    let key = Key::new(password.as_str(), &salt);

    debug_println!("Key created");

    let db = AppDataDb::get_or_init(network, key.clone())
        .await
        .map_err(|err| AppError::NonFatal(types::Notification::Info(err.to_string())))?;

    debug_println!("Database successfully loaded");

    let target_hash = db
        .get_db_password_hash()
        .await
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    if password_hash == target_hash {
        debug_println!("Correct password");
        IconsDb::load(network, key)
            .map_err(|err| AppError::Fatal(err.to_string()))
            .await?;
        Ok(())
    } else {
        Err(AppError::NonFatal(types::Notification::Info(
            "Incorrect Password".to_string(),
        )))
    }
}
