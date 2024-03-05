use thiserror::Error;

use self::radixrequest::RadixDltRequestBuilder;

pub mod radixrequest;




#[derive(Debug, Error)]
pub enum ComsError {
    #[error("{0}")]
    RadixDltError(#[from] radixrequest::RadixDltRequestError),
}


#[derive(Debug)]
pub struct Coms {
    pub radixdlt_request_builder: RadixDltRequestBuilder,
}

impl Coms {
    pub fn new() -> Result<Self, ComsError> {
        let radixdlt_request_builder = RadixDltRequestBuilder::new()?;

        Ok(Self {
            radixdlt_request_builder,
        })
    }

    pub fn radix_request_client(&self) -> reqwest::Client {
        self.radixdlt_request_builder.client()
    }
}

