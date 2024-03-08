use bip39::{Mnemonic, Language, MnemonicType, Seed};
use ed25519_dalek_fiat::{SecretKey, PublicKey};
use scrypto::prelude::radix_engine_common::prelude::*;
use types::{crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair, Password}, Account, AccountAddress, Network};

pub fn create_new_wallet() -> Result<(),std::io::Error> {
    let mnemonic = Mnemonic::new(MnemonicType::Words24, Language::English);

    create_wallet_from_mnemonic(mnemonic)?;
    Ok(())
}


pub fn create_wallet_from_mnemonic(mnemonic: Mnemonic) -> Result<(), std::io::Error> {

    let seed = Seed::new(&mnemonic, "");
    // let derivation_path = "m/44'/1022'/1'/525'/1460'/0'";
    let indexes = [44,1022,1,525,1460,0];

    let slip_10 = slip10_ed25519::derive_ed25519_private_key(seed.as_bytes(), indexes.as_slice());

    //from_bytes() will only fail if the &[u8] is not 32 bytes long, this will always be and so will never throw an error
    let private_key = SecretKey::from_bytes(&slip_10).unwrap();
    let public_key = PublicKey::from(&private_key);

    // let pub_key_bytes = public_key.as_bytes();
    let ed25519_public_key = scrypto::crypto::Ed25519PublicKey(public_key.as_bytes().to_owned());
    let address = ComponentAddress::virtual_account_from_public_key(&ed25519_public_key);

    let bech32 = AddressBech32Encoder::new(&NetworkDefinition::mainnet());
    let _bech32_address = bech32.encode(address.to_vec().as_slice()).unwrap();

    Ok(())
}


pub fn create_account_from_mnemonic(mnemonic: &Mnemonic, account_index: u32, account_name: String, network: Network) -> Account {

    let (keypair, path) = Ed25519KeyPair::from_mnemonic(
        mnemonic,
        account_index,
        network,
        Bip32Entity::Account,
        Bip32KeyKind::TransactionSigning,
    );

    let radixdlt_pub_key = keypair.radixdlt_public_key();
    let account_address = keypair.bech32_address();
    let account_address = AccountAddress::try_from(account_address.as_bytes())
        .unwrap_or_else(|err| unreachable!("{}:{} Invalid account address: {err}", module_path!(), line!()));

    let account = Account::new(
        0,
        account_name,
        network,
        path,
        account_address,
        radixdlt_pub_key,
    );
    account
}