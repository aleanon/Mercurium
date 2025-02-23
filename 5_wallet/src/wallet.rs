pub(crate) mod resource_data;
pub(crate) mod initial;
pub(crate) mod locked;
pub(crate) mod unlocked;
pub(crate) mod wallet_data;

use wallet_data::WalletData;

use crate::app_state::WalletState;



pub struct Wallet<State: WalletState> {
    state: State,
    wallet_data: WalletData
}

impl<State> Wallet<State> where State: WalletState {
    pub fn new(state: State, wallet_data: WalletData) -> Self {
        Self { state, wallet_data }
    }
   
}
