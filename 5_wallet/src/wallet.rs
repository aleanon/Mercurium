pub(crate) mod resource_data;
pub(crate) mod wallet_setup;
pub(crate) mod locked;
pub(crate) mod unlocked;
pub(crate) mod wallet_data;

use types::AppSettings;
use wallet_data::WalletData;



// pub enum Wallet {
//     Initial(InnerWallet<Setup>),
//     Locked(InnerWallet<Locked>),
//     Unlocked(InnerWallet<Unlocked>),
//     Error(AppError),
// }

// impl Wallet {
//     pub fn new() -> Self {
//         let settings = handles::app_settings::get_app_settings();

//         match handles::statics::initialize_statics::initialize_statics(settings.network) {
//             Err(err) => Self::Error(err),
//             Ok(_) => {
//                 if AppDataDb::exists(settings.network) {
//                     Wallet::Locked( InnerWallet::new(Locked::new(), WalletData::new(settings)))
//                 } else {
//                     Wallet::Initial(InnerWallet::new(Setup::new(), WalletData::new(settings)))
//                 }
//             }
//         }
//     }
// }

pub trait WalletState {}

#[derive(Debug, Clone)]
pub struct Wallet<State: WalletState> {
    state: State,
    wallet_data: WalletData
}

impl<State> Wallet<State> where State: WalletState {
    pub fn new(state: State, wallet_data: WalletData) -> Self {
        Self {state, wallet_data}
    }

    pub fn settings(&self) -> &AppSettings {
        &self.wallet_data.settings
    }
}
