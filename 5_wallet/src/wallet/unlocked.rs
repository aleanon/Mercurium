use crate::app_state::WalletState;

use super::{locked::Locked, wallet_data::WalletData, Wallet};

#[derive(Clone)]
pub struct Unlocked;



impl WalletState for Unlocked{}

impl Wallet<Unlocked> {
    pub fn logout(self) -> Wallet<Locked> {
        Wallet {state: Locked::new(), wallet_data: self.wallet_data}
    }

    pub fn wallet_data_mut(&mut self) -> &mut WalletData {
        &mut self.wallet_data
    } 

    pub fn wallet_data(&self) -> &WalletData {
        &self.wallet_data
    }
}