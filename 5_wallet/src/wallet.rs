pub(crate) mod resource_data;
pub(crate) mod wallet_setup;
pub(crate) mod locked;
pub(crate) mod unlocked;
pub(crate) mod wallet_data;

use std::str::FromStr;

use deps::bip39::Mnemonic;
use types::{address::AccountAddress, crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair}, debug_info, Account, Network, UnwrapUnreachable};
use wallet_data::WalletData;

use crate::settings::Settings;


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

    pub fn settings(&self) -> &Settings{
        &self.wallet_data.settings
    }

}

pub(crate) fn create_multiple_accounts_from_mnemonic<T: FromIterator<Account>>(
    mnemonic: &Mnemonic,
    password: Option<&str>,
    mut start_id: usize,
    account_index: u32,
    number_of_accounts: u32,
    network: Network,
) -> T {
    let end_index = account_index + number_of_accounts;

    (account_index..end_index).map(|i| {
        let account = create_account_from_mnemonic(
            mnemonic,
            password,
            start_id,
            i,
            String::new(),
            network,
        );
        start_id += 1;
        account
    })
    .collect()
}

pub(crate) fn create_account_from_mnemonic(
    mnemonic: &Mnemonic,
    password: Option<&str>,
    id: usize,
    account_index: u32,
    account_name: String,
    network: Network,
) -> Account {
    let (keypair, path) = Ed25519KeyPair::new(
        mnemonic,
        password,
        account_index,
        network,
        Bip32Entity::Account,
        Bip32KeyKind::TransactionSigning,
    );

    let radixdlt_pub_key = keypair.radixdlt_public_key();
    let account_address = keypair.bech32_address();
    let account_address = AccountAddress::from_str(account_address.as_str())
        .unwrap_unreachable(debug_info!("Invalid account address"));

    let account = Account::new(
        id,
        account_name,
        network,
        path,
        account_address,
        radixdlt_pub_key,
    );
    account
}