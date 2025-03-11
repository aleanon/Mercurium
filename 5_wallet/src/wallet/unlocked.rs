use crate::app_state::WalletState;

use super::{locked::Locked, wallet_data::WalletData, InnerWallet};


pub struct Unlocked;



impl WalletState for Unlocked{}

impl InnerWallet<Unlocked> {
    pub fn logout(self) -> InnerWallet<Locked> {
        InnerWallet {state: Locked::new(), wallet_data: self.wallet_data}
    }

    pub fn wallet_data_mut(&mut self) -> &mut WalletData {
        &mut self.wallet_data
    } 

    pub fn wallet_data(&self) -> &WalletData {
        &self.wallet_data
    }
}