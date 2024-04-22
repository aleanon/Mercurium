use reqwest::{
    header::{HeaderMap, ACCEPT, ACCEPT_ENCODING, CONNECTION, CONTENT_TYPE, USER_AGENT},
    Client,
};
use std::ops::Deref;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RadixDltRequestError {
    #[error("Unable to create request header, source: {0}")]
    FailedToParseHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Unable to build reqwest Client, source: {0}")]
    FailedToBuildClient(reqwest::Error),
    #[error("Request failed, source: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Unable to decode response, source: {0}")]
    DecodeError(#[from] std::io::Error),
    #[error("Unable to parse response from json, source: {0}")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct RadixDltRequestClient(Client);

impl RadixDltRequestClient {
    pub fn new() -> Result<Self, RadixDltRequestError> {
        let mut headers = HeaderMap::with_capacity(5);
        headers.insert(CONTENT_TYPE, "application/json".parse()?);
        headers.insert(USER_AGENT, "PostmanRuntime/7.34.0".parse()?);
        headers.insert(ACCEPT, "*/*".parse()?);
        headers.insert(ACCEPT_ENCODING, "gzip".parse()?);
        headers.insert(CONNECTION, "keep-alive".parse()?);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self(client))
    }
}

impl Deref for RadixDltRequestClient {
    type Target = reqwest::Client;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
