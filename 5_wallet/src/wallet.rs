pub(crate) mod locked;
pub mod login;
pub(crate) mod resource_data;
pub(crate) mod unlocked;
pub(crate) mod wallet_data;
pub(crate) mod wallet_setup;

use std::str::FromStr;
use std::sync::Arc;

use deps::bip39::Mnemonic;
use types::{
    Account, AppError, Network, Persona, PersonaData, UnwrapUnreachable,
    address::AccountAddress,
    crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair, Password},
    debug_info,
};
use wallet_data::WalletData;

use crate::settings::Settings;

/// Decrypts and returns the wallet mnemonic (and its optional seed password) using the user's
/// login password. Moved here from `handles::wallet::get_mnemonic`.
///
/// This is the signing-session primitive: callers re-decrypt the mnemonic for each signing
/// operation and drop it immediately (it is zeroized on drop) rather than caching it. Decryption
/// derives a key with a high iteration count, so run it off the UI thread.
pub fn get_decrypted_mnemonic(
    secrets: &Arc<dyn secrets_store::SecretsStore>,
    password: &Password,
) -> Result<(Mnemonic, Password), AppError> {
    let encrypted = secrets
        .get_encrypted_mnemonic()
        .map_err(|err| AppError::Fatal(err.to_string()))?;

    encrypted
        .decrypt_mnemonic(password)
        .map_err(|err| AppError::NonFatal(types::Notification::Info(err.to_string())))
}

pub trait WalletState {}

#[derive(Debug, Clone)]
pub struct Wallet<State: WalletState> {
    state: State,
    wallet_data: WalletData,
}

impl<State> Wallet<State>
where
    State: WalletState,
{
    pub fn new(state: State, wallet_data: WalletData) -> Self {
        Self { state, wallet_data }
    }

    pub fn settings(&self) -> &Settings {
        &self.wallet_data.settings
    }
}

pub(crate) fn create_multiple_accounts_from_mnemonic<T: FromIterator<Account>>(
    mnemonic: &Mnemonic,
    password: Option<&str>,
    mut start_id: i64,
    account_index: u32,
    number_of_accounts: u32,
    network: Network,
) -> T {
    let end_index = account_index + number_of_accounts;

    (account_index..end_index)
        .map(|i| {
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

/// Derives a new persona (identity) from the wallet mnemonic at the given index.
///
/// Mirrors [`create_account_from_mnemonic`] but uses the `Identity` BIP-32 entity, so the
/// derived virtual address is an `identity_...` address. The returned persona starts with empty
/// [`PersonaData`]; the user fills in what they want to share with dApps afterwards.
pub fn create_persona_from_mnemonic(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    id: i64,
    identity_index: u32,
    label: String,
    network: Network,
) -> Persona {
    let (keypair, path) = Ed25519KeyPair::new(
        mnemonic,
        seed_password,
        identity_index,
        network,
        Bip32Entity::Identity,
        Bip32KeyKind::TransactionSigning,
    );

    let identity_address = keypair.bech32_address();
    let public_key = keypair.radixdlt_public_key();

    Persona::new(
        id,
        label,
        network,
        identity_address,
        public_key,
        path,
        PersonaData::default(),
    )
}

/// Reconstructs the identity (persona) signing key pair, e.g. for ROLA.
///
/// Like [`signing_keypair_for_account`] but for the `Identity` entity.
pub fn signing_keypair_for_persona(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    persona: &Persona,
) -> Ed25519KeyPair {
    let (keypair, _) = Ed25519KeyPair::new(
        mnemonic,
        seed_password,
        persona.derivation_index(),
        persona.network,
        Bip32Entity::Identity,
        Bip32KeyKind::TransactionSigning,
    );
    keypair
}

/// Reconstructs the transaction-signing key pair for an existing account.
///
/// An [`Account`] stores its derivation index and network, and every account uses the
/// `Account` entity with the `TransactionSigning` key kind, so the key pair can be
/// re-derived deterministically from the wallet mnemonic at signing time. The returned
/// key pair exposes [`Ed25519KeyPair::radix_private_key`] for use with a transaction builder.
pub fn signing_keypair_for_account(
    mnemonic: &Mnemonic,
    seed_password: Option<&str>,
    account: &Account,
) -> Ed25519KeyPair {
    let (keypair, _) = Ed25519KeyPair::new(
        mnemonic,
        seed_password,
        account.derivation_index(),
        account.network,
        Bip32Entity::Account,
        Bip32KeyKind::TransactionSigning,
    );
    keypair
}

pub(crate) fn create_account_from_mnemonic(
    mnemonic: &Mnemonic,
    password: Option<&str>,
    id: i64,
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

#[cfg(test)]
mod persona_tests {
    use super::*;
    use deps::bip39::{Language, Mnemonic};

    fn mnemonic() -> Mnemonic {
        Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap()
    }

    #[test]
    fn derives_persona_with_identity_address() {
        let persona = create_persona_from_mnemonic(
            &mnemonic(),
            None,
            0,
            0,
            "Main Persona".to_string(),
            Network::Mainnet,
        );

        assert!(persona.identity_address.starts_with("identity_rdx"));
        assert_eq!(persona.derivation_index(), 0);
        assert_eq!(persona.derivation_path()[3], Bip32Entity::Identity.path_index());

        // The persona key must reproduce the same public key when re-derived for signing.
        let keypair = signing_keypair_for_persona(&mnemonic(), None, &persona);
        assert_eq!(keypair.radixdlt_public_key(), persona.public_key);
    }
}
