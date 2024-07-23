use bip39::{Mnemonic, Seed};
use ed25519_dalek_fiat::{PublicKey, SecretKey};
use scrypto::{
    address::AddressBech32Encoder, crypto::Ed25519PublicKey, network::NetworkDefinition,
    types::ComponentAddress,
};
use slip10_ed25519::derive_ed25519_private_key;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable, Network};

use super::Password;

const BIP32_LEAD_WORD: u32 = 44; // 0
const BIP32_COIN_TYPE_RADIX: u32 = 1022; // 1
const BIP32_NETWORK_ID_MAINNET: u32 = 1; //2
const BIP32_NETWORK_ID_STOKENET: u32 = 2; //2
const BIP32_ENTITY_ACCOUNT: u32 = 525; // 3
const BIP32_ENTITY_IDENTITY: u32 = 618; //3
const BIP32_KEY_KIND_TRANSACTION_SIGNING: u32 = 1460; // 4
const BIP32_KEY_KIND_AUTHENTICATION_SIGNING: u32 = 1678; // 4
const BIP32_KEY_KIND_MESSAGE_ENCRYPTION: u32 = 1391; // 4

#[derive(Debug)]
pub enum Bip32Entity {
    Account,
    Identity,
}

#[derive(Debug)]
pub enum Bip32KeyKind {
    TransactionSigning,
    AuthenticationSigning,
    MessageEncryption,
}

///A key-pair from the dalek_ed25519_fiat crate.
#[derive(Debug, ZeroizeOnDrop)]
pub struct Ed25519KeyPair {
    secret_key: SecretKey,
    #[zeroize(skip)]
    public_key: PublicKey,
    #[zeroize(skip)]
    network: Network,
    #[zeroize(skip)]
    entity: Bip32Entity,
    #[zeroize(skip)]
    key_kind: Bip32KeyKind,
}

impl Ed25519KeyPair {
    pub fn new(
        mnemonic: &Mnemonic,
        password: Option<&str>,
        index: u32,
        network: Network,
        entity: Bip32Entity,
        key_kind: Bip32KeyKind,
    ) -> (Self, [u32; 6]) {
        let seed = Seed::new(mnemonic, password.unwrap_or(""));

        let network_id = match network {
            Network::Mainnet => BIP32_NETWORK_ID_MAINNET,
            Network::Stokenet => BIP32_NETWORK_ID_STOKENET,
        };

        let bip32_entity = match entity {
            Bip32Entity::Account => BIP32_ENTITY_ACCOUNT,
            Bip32Entity::Identity => BIP32_ENTITY_IDENTITY,
        };

        let bip32_key_kind = match key_kind {
            Bip32KeyKind::TransactionSigning => BIP32_KEY_KIND_TRANSACTION_SIGNING,
            Bip32KeyKind::AuthenticationSigning => BIP32_KEY_KIND_AUTHENTICATION_SIGNING,
            Bip32KeyKind::MessageEncryption => BIP32_KEY_KIND_MESSAGE_ENCRYPTION,
        };

        //The starting "m/" is omitted from the derivation path with this implementation
        let derivation_path = [
            BIP32_LEAD_WORD,
            BIP32_COIN_TYPE_RADIX,
            network_id,
            bip32_entity,
            bip32_key_kind,
            index,
        ];

        //The derive_ed25519_private_key function treats all indexes as hardened
        let mut priv_key = derive_ed25519_private_key(seed.as_bytes(), derivation_path.as_slice());

        //SecretKey::from_bytes() will only fail if the &[u8] is not of length 32 which it always will be, so unwrap is called
        let secret_key = SecretKey::from_bytes(&priv_key)
            .unwrap_unreachable(debug_info!("Invalid secret key length"));

        let public_key = PublicKey::from(&secret_key);

        priv_key.zeroize();

        (
            Self {
                secret_key,
                public_key,
                network,
                entity,
                key_kind,
            },
            derivation_path,
        )
    }

    pub fn radixdlt_public_key(&self) -> Ed25519PublicKey {
        Ed25519PublicKey(self.public_key.to_bytes().to_owned())
    }

    pub fn bech32_address(&self) -> String {
        let network_definition = match self.network {
            Network::Mainnet => NetworkDefinition::mainnet(),
            Network::Stokenet => NetworkDefinition::stokenet(),
        };

        let virtual_account_address =
            ComponentAddress::virtual_account_from_public_key(&self.radixdlt_public_key());

        let encoder = AddressBech32Encoder::new(&network_definition);
        //We know the data we pass to encode is of type ComponentAddress, this will always be a valid Bech32 address so we call unwrap
        let address = encoder
            .encode(virtual_account_address.as_ref())
            .expect("Unreachable error, invalid Bech32 address");

        address
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bip39::Language;

    #[test]
    fn test_address_from_mnemonic_with_index_mainnet() {
        let mnemonic = Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis", 
            Language::English
        ).unwrap();
        let (keypair, _) = Ed25519KeyPair::new(
            &mnemonic,
            None,
            0,
            Network::Mainnet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );
        let account_address = keypair.bech32_address();
        println!("account_address: {}", account_address);
        let (keypair2, _) = Ed25519KeyPair::new(
            &mnemonic,
            None,
            1,
            Network::Mainnet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );
        let account_address2 = keypair2.bech32_address();
        println!("account_address2: {}", account_address2);

        assert_eq!(
            account_address.as_str(),
            "account_rdx128ykx9agh0maq8nw6h6pzmltmaexts0xf24sledqp44x5cdec0uqjj"
        );
        assert_eq!(
            account_address2.as_str(),
            "account_rdx12xn8d9ykr8pmch33v0q66vhpvt98afalg0tfgctrqenkly96sgdx5n"
        );
    }

    #[test]
    fn test_address_from_mnemonic_with_index_stokenet() {
        let mnemonic = Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis", 
            Language::English
        ).unwrap();
        let (keypair, _) = Ed25519KeyPair::new(
            &mnemonic,
            None,
            0,
            Network::Stokenet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );
        let account_address = keypair.bech32_address();
        println!("account_address: {}", account_address);
        let (keypair2, _) = Ed25519KeyPair::new(
            &mnemonic,
            None,
            1,
            Network::Stokenet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );
        let account_address2 = keypair2.bech32_address();
        println!("account_address2: {}", account_address2);

        assert_eq!(
            account_address.as_str(),
            "account_tdx_2_12y0kpp2nhn8f36gt2ppqmxeltj6n2r446s0jlh4l7yxttpfeahjn66"
        );
        assert_eq!(
            account_address2.as_str(),
            "account_tdx_2_12866llg04px7q2wee02yxcxtdwgtpzdc8n75fermd070u64t98jtnj"
        );
    }
}
