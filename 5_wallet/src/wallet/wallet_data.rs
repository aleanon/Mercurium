use types::Network;

use crate::settings::Settings;

use super::resource_data::ResourceData;







pub struct WalletData {
    pub resource_data: ResourceData,
    pub settings: Settings,
}


impl WalletData {
    pub fn new(settings: Settings) -> Self {
        Self { resource_data: ResourceData::new(), settings }
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn network(&self) -> Network {
        self.settings.network()
    }

    pub fn set_network(&mut self, network: Network) {
        self.settings.network = network;
    }


}