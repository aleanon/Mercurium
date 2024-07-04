use std::error::Error;

use store::IconCache;
use types::{
    app_error::ErrorString,
    app_path::{self, AppPath, APP_PATH},
    network, AccountAddress, AppError, AppPathInner, Network,
};

pub fn initialize_statics(network: Network) -> Result<(), AppError> {
    match AppPathInner::new() {
        Err(err) => return Err(AppError::Fatal(Box::new(err))),
        Ok(_) => AppPath::get(),
    };

    radix_gateway_sdk::Client::new(network.into(), None, None)
        .map_err(|err| AppError::Fatal(Box::new(err)))?;

    // address regexes

    Ok(())
}
