use super::derivation_path_indexes::{
    BIP32_KEY_KIND_AUTHENTICATION_SIGNING, BIP32_KEY_KIND_MESSAGE_ENCRYPTION,
    BIP32_KEY_KIND_TRANSACTION_SIGNING,
};

#[derive(Debug)]
pub enum Bip32KeyKind {
    TransactionSigning,
    AuthenticationSigning,
    MessageEncryption,
}

impl Bip32KeyKind {
    pub fn path_index(&self) -> u32 {
        match self {
            Bip32KeyKind::TransactionSigning => BIP32_KEY_KIND_TRANSACTION_SIGNING,
            Bip32KeyKind::AuthenticationSigning => BIP32_KEY_KIND_AUTHENTICATION_SIGNING,
            Bip32KeyKind::MessageEncryption => BIP32_KEY_KIND_MESSAGE_ENCRYPTION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_kind_path_indexes() {
        assert_eq!(Bip32KeyKind::TransactionSigning.path_index(), 1460);
        assert_eq!(Bip32KeyKind::AuthenticationSigning.path_index(), 1678);
        assert_eq!(Bip32KeyKind::MessageEncryption.path_index(), 1391);
    }
}
