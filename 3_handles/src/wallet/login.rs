use debug_print::debug_println;
use store::AppDataDb;
use types::{crypto::Password, AppError, Network};

pub async fn perform_login_check(network: Network, password: Password) -> Result<(), AppError> {
    let salt = crate::credentials::get_db_encryption_salt()?;
    let password_hash = password.derive_db_encryption_key_hash_from_salt(&salt);

    let key = password.derive_db_encryption_key_from_salt(&salt);

    debug_println!("Key created");

    let db = AppDataDb::get_or_init(network, key)
        .await
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    debug_println!("Database successfully loaded");

    let target_hash = db
        .get_db_password_hash()
        .await
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    if password_hash == target_hash {
        debug_println!("Correct password");
        return Ok(());
    } else {
        return Err(AppError::NonFatal(types::Notification::Info(
            "Incorrect Password".to_string(),
        )));
    }
}
