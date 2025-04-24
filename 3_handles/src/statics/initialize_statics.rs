use deps::*;

use std::str::FromStr;

use types::{
    address::{AccountAddress, ResourceAddress},
    AppError, AppPathInner, Network,
};

pub fn initialize_statics(network: Network) -> Result<(), AppError> {
    match AppPathInner::new() {
        Err(err) => return Err(AppError::Fatal(err.to_string())),
        Ok(_) => {},
    };

    radix_gateway_sdk::Client::new(network.into(), None, None)
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    // address regexes
    AccountAddress::from_str("some_invalid_address_to_initialize_the_regex_statics").ok();
    ResourceAddress::from_str("some_invalid_address_to_initialize_the_regex_statics").ok();

    Ok(())
}
