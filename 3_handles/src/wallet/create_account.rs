use bip39::Mnemonic;
use types::{
    address::AccountAddress,
    crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair},
    Account, Network,
};

pub fn create_account_from_mnemonic(
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
    let account_address =
        AccountAddress::try_from(account_address.as_bytes()).unwrap_or_else(|err| {
            unreachable!(
                "{}:{} Invalid account address: {err}",
                module_path!(),
                line!()
            )
        });

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
