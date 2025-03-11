use types::{AppSettings, Network};


use super::resource_data::ResourceData;







pub struct WalletData {
    pub resource_data: ResourceData,
    pub settings: AppSettings,
}


impl WalletData {
    pub fn new(settings: AppSettings) -> Self {
        Self { resource_data: ResourceData::new(), settings }
    }

    pub fn get_settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn network(&self) -> Network {
        self.settings.network
    }

    pub fn set_network(&mut self, network: Network) {
        self.settings.network = network;
    }


}