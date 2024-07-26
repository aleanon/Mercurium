use types::{AppError, AppPath, AppPathInner, Network};

pub fn initialize_statics(network: Network) -> Result<(), AppError> {
    match AppPathInner::get_application_root_directory() {
        Err(err) => return Err(AppError::Fatal(err.to_string())),
        Ok(_) => AppPath::get(),
    };

    radix_gateway_sdk::Client::new(network.into(), None, None)
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    // address regexes

    Ok(())
}
