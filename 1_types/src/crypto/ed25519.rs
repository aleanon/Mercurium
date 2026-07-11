use deps::*;

use std::fmt::Debug;

use bip39::{Mnemonic, Seed};
use ed25519_dalek_fiat::{PublicKey, SecretKey};
use scrypto::{address::AddressBech32Encoder, crypto::Ed25519PublicKey, types::ComponentAddress};
use slip10_ed25519::derive_ed25519_private_key;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use radix_common::crypto::{Ed25519PrivateKey, Ed25519Signature, Hash};

use crate::{Network, debug_info, unwrap_unreachable::UnwrapUnreachable};

use super::{
    bip32_entity::Bip32Entity,
    bip32_key_kind::Bip32KeyKind,
    derivation_path_indexes::{BIP32_COIN_TYPE_RADIX, BIP32_LEAD_WORD},
};

///A key-pair from the dalek_ed25519_fiat crate.
#[derive(ZeroizeOnDrop)]
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

        //The starting "m/" is omitted from the derivation path with this implementation
        let derivation_path = [
            BIP32_LEAD_WORD,
            BIP32_COIN_TYPE_RADIX,
            network.id(),
            entity.path_index(),
            key_kind.path_index(),
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

    /// Returns the radix-common Ed25519 private key for this account.
    ///
    /// This type implements `radix_transactions::signing::Signer`, so it can be passed
    /// directly to a `TransactionBuilder`'s `sign`/`notarize` methods. The intermediate
    /// 32-byte buffer is wrapped in `Zeroizing` so the secret is wiped from the stack.
    pub fn radix_private_key(&self) -> Ed25519PrivateKey {
        let bytes = Zeroizing::new(self.secret_key.to_bytes());
        Ed25519PrivateKey::from_bytes(bytes.as_ref())
            .unwrap_unreachable(debug_info!("Invalid ed25519 private key length"))
    }

    /// Signs a pre-computed message hash (e.g. a transaction intent hash), returning a
    /// radix Ed25519 signature. Transaction building normally goes through
    /// [`Self::radix_private_key`] + the builder; this is for standalone signing such as ROLA.
    pub fn sign_hash(&self, hash: &Hash) -> Ed25519Signature {
        self.radix_private_key().sign(hash)
    }

    pub fn bech32_address(&self) -> String {
        let network_definition = self.network.definition();

        // The virtual (pre-allocated) global address depends on the entity kind: an account
        // key derives an `account_` address, an identity (persona) key an `identity_` address.
        let public_key = self.radixdlt_public_key();
        let virtual_address = match &self.entity {
            Bip32Entity::Account => {
                ComponentAddress::preallocated_account_from_public_key(&public_key)
            }
            Bip32Entity::Identity => {
                ComponentAddress::preallocated_identity_from_public_key(&public_key)
            }
        };

        // TODO! Lag en lazy static for address encoder
        let encoder = AddressBech32Encoder::new(&network_definition);
        //We know the data we pass to encode is of type ComponentAddress, this will always be a valid Bech32 address so we call unwrap
        

        encoder
            .encode(virtual_address.as_ref())
            .unwrap_unreachable(debug_info!("invalid Bech32 address"))
    }
}

impl Debug for Ed25519KeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ed25519KeyPair {{ secret_key: *, public_key: {:?}, network: {:?}, entity: {:?}, key_kind: {:?} }}",
            self.public_key, self.network, self.entity, self.key_kind
        )
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

    #[test]
    fn test_identity_address_differs_from_account_address() {
        let mnemonic = Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap();

        let (account_keypair, _) = Ed25519KeyPair::new(
            &mnemonic,
            None,
            0,
            Network::Mainnet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );
        let (identity_keypair, _) = Ed25519KeyPair::new(
            &mnemonic,
            None,
            0,
            Network::Mainnet,
            Bip32Entity::Identity,
            Bip32KeyKind::TransactionSigning,
        );

        let account_address = account_keypair.bech32_address();
        let identity_address = identity_keypair.bech32_address();

        assert!(account_address.starts_with("account_rdx"));
        assert!(
            identity_address.starts_with("identity_rdx"),
            "got: {identity_address}"
        );
        assert_ne!(account_address, identity_address);
    }

    #[test]
    fn test_sign_and_verify_hash() {
        use radix_common::crypto::{hash, verify_ed25519};

        let mnemonic = Mnemonic::from_phrase(
            "toward point obtain quit degree route beauty magnet hidden cereal reform increase limb measure guide skirt nominee faint shoulder win deal april error axis",
            Language::English,
        )
        .unwrap();

        let (keypair, _) = Ed25519KeyPair::new(
            &mnemonic,
            None,
            0,
            Network::Mainnet,
            Bip32Entity::Account,
            Bip32KeyKind::TransactionSigning,
        );

        // The radix private key's public key must match the address-deriving public key.
        let radix_pub = keypair.radix_private_key().public_key();
        assert_eq!(radix_pub, keypair.radixdlt_public_key());

        // Sign a hash and verify it against that public key.
        let message_hash = hash(b"mercurium transaction intent");
        let signature = keypair.sign_hash(&message_hash);
        assert!(verify_ed25519(message_hash, &radix_pub, &signature));

        // A different message must not verify against the same signature.
        let other_hash = hash(b"a different message");
        assert!(!verify_ed25519(other_hash, &radix_pub, &signature));
    }
}
