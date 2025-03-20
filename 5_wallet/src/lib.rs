mod app_state;
mod wallet_keys_and_salt;
pub mod wallet;
pub mod error;
pub mod settings;


pub use wallet::Wallet;
pub use wallet::wallet_setup::setup::Setup;
pub use wallet::wallet_setup::setup_error::SetupError;
pub use wallet::unlocked::Unlocked;
pub use wallet::locked::Locked;