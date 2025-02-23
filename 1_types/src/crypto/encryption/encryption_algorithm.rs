use ring::aead::{Algorithm, AES_128_GCM, AES_256_GCM, CHACHA20_POLY1305};

#[allow(non_camel_case_types)]
pub enum EncryptionAlgorithm {
    Aes_128_GCM,
    Aes_256_GCM,
    ChaCha20_Poly1305,
}

pub const fn encryption_key_length(algorithm: EncryptionAlgorithm) -> usize {
    match algorithm {
        EncryptionAlgorithm::Aes_128_GCM => 16,
        EncryptionAlgorithm::Aes_256_GCM => 32,
        EncryptionAlgorithm::ChaCha20_Poly1305 => 32,
    }
}

impl Into<&'static Algorithm> for EncryptionAlgorithm {
    fn into(self) -> &'static Algorithm {
        match self {
            EncryptionAlgorithm::Aes_128_GCM => &AES_128_GCM,
            EncryptionAlgorithm::Aes_256_GCM => &AES_256_GCM,
            EncryptionAlgorithm::ChaCha20_Poly1305 => &CHACHA20_POLY1305,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_constant_key_lengths() {
        assert_eq!(encryption_key_length(EncryptionAlgorithm::Aes_128_GCM), AES_128_GCM.key_len());
        assert_eq!(encryption_key_length(EncryptionAlgorithm::Aes_256_GCM), AES_256_GCM.key_len());
        assert_eq!(encryption_key_length(EncryptionAlgorithm::ChaCha20_Poly1305), CHACHA20_POLY1305.key_len());
    }
}