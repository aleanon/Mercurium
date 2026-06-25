mod http_icon_provider;
mod port;
mod resize;

pub use http_icon_provider::HttpIconProvider;
pub use port::IconProvider;
pub use resize::resize_standard_dimensions_from_bytes;

use std::collections::{BTreeMap, HashMap};

use types::address::ResourceAddress;

/// Convenience: download + resize a single icon via the default HTTP provider.
pub async fn fetch_icon(url: &str) -> Option<Vec<u8>> {
    HttpIconProvider.fetch_icon(url).await
}

/// Convenience: download + resize a batch of icons via the default HTTP provider.
pub async fn fetch_icons(
    urls: BTreeMap<ResourceAddress, String>,
) -> HashMap<ResourceAddress, (Vec<u8>, Vec<u8>)> {
    HttpIconProvider.fetch_icons(urls).await
}
