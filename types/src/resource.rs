use crate::{MetaData, ResourceAddress};

pub struct Resource {
    pub address: ResourceAddress,
    pub name: String,
    pub symbol: String,
    pub total_supply: String,
    pub description: String,
    pub last_updated: usize,
    pub metadata: MetaData,
}
