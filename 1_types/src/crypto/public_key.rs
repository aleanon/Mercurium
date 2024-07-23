use scrypto::crypto::{Ed25519PublicKey, Secp256k1PublicKey};

pub enum PublicKey {
    Ed25519(Ed25519PublicKey),
    Secp256k1(Secp256k1PublicKey),
}

impl Into<PublicKeyType> for &PublicKey {
    fn into(self) -> PublicKeyType {
        match self {
            PublicKey::Ed25519(_) => PublicKeyType::Ed25519,
            PublicKey::Secp256k1(_) => PublicKeyType::Secp256k1,
        }
    }
}

pub enum PublicKeyType {
    Ed25519,
    Secp256k1,
}

