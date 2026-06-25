use std::collections::{BTreeMap, HashMap};

use async_trait::async_trait;
use types::address::ResourceAddress;

/// Port for acquiring resource icon images: download from a URL (resource metadata points at
/// arbitrary CDNs / IPFS gateways) and resize to the variants the wallet displays. This is
/// *acquisition + transform* only — persistence lives in the data store.
#[async_trait]
pub trait IconProvider {
    /// Download a single icon and resize it to standard display dimensions.
    async fn fetch_icon(&self, url: &str) -> Option<Vec<u8>>;

    /// Download a batch of icons, returning the `(small, standard)` encoded variants per resource.
    async fn fetch_icons(
        &self,
        urls: BTreeMap<ResourceAddress, String>,
    ) -> HashMap<ResourceAddress, (Vec<u8>, Vec<u8>)>;
}
