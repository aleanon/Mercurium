use std::num::NonZeroU32;

use bip39::Mnemonic;
use ring::pbkdf2;
use ring::aead::{NonceSequence, NONCE_LEN, Nonce, BoundKey, AES_256_GCM, UnboundKey, Aad, OpeningKey};
use ring::rand::{SystemRandom, SecureRandom};
use thiserror::Error;
use types::crypto::{Password, Salt};
use zeroize::Zeroize;
use serde::{Serialize,Deserialize};
use std::error::Error as StdError;


const ITERATIONS:u32 = 1000000; 

#[derive(Error, Debug)]
pub enum EncryptedMnemonicError{
    #[error("Failed to create random value")]
    FailedToCreateRandomValue,
    #[error("Failed to create unbound key")]
    FailedToCreateUnboundKey,
    #[error("Failed to encrypt mnemonic")]
    FailedToEncryptMnemonic,
    #[error("Failed to decrypt mnemonic")]
    FailedToDecryptMnemonic,
    #[error("Failed to parse, invalid utf-8")]
    FailedToParseInvalidUtf8,
    #[error("Failed to parse EncryptedMnemonic")]
    FailedToParseEncryptedMnemonic,
    #[error("Failed to save mnemonic, {0}")]
    FailedToSaveCredentials(Box<dyn StdError>),
    #[error("Failed to retrieve Credentials: {0}")]
    FailedToRetrieveCredentials(Box<dyn StdError>),
    #[error("Failed to delete Credentials: {0}")]
    FailedToDeleteCredentials(Box<dyn StdError>),
    #[error("Failed to construct mnemonic")]
    FailedToConstructMnemonic,
}


struct MnemonicNonceSequence(Nonce);

impl MnemonicNonceSequence {
    pub fn new() -> Result<Self, EncryptedMnemonicError> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        SystemRandom::new().fill(&mut nonce_bytes)
        .map_err(|_| EncryptedMnemonicError::FailedToCreateRandomValue)?;
    
    Ok(Self(Nonce::assume_unique_for_key(nonce_bytes)))
}

    pub fn with_nonce(nonce: &Nonce) -> Self {
        Self(Nonce::assume_unique_for_key(nonce.as_ref().clone()))
    }

    pub fn get_current(&self) -> [u8; NONCE_LEN] {
        self.0.as_ref().clone()
    }
}

impl NonceSequence for MnemonicNonceSequence {
    fn advance(&mut self) -> Result<ring::aead::Nonce, ring::error::Unspecified> {
        let nonce = Nonce::assume_unique_for_key(self.get_current());
        Ok(nonce)

        // let nonce = Nonce::assume_unique_for_key(self.get_current());
        // let mut new_nonce = [0u8; NONCE_LEN];
        // SystemRandom::new().fill(&mut new_nonce)?;
        // self.0 = Nonce::assume_unique_for_key(new_nonce);
        // Ok(nonce)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedMnemonic{
    cypher_text: Vec<u8>,
    salt: Salt,
    nonce_bytes: [u8;NONCE_LEN],
}

impl EncryptedMnemonic {

    pub fn new(mnemonic: &Mnemonic, password: &Password) -> Result<Self, EncryptedMnemonicError> {
        let mut mnemonic:Vec<u8> = mnemonic.phrase().into();
        let (mut key, salt) = password.derive_new_mnemonic_encryption_key()
            .map_err(|err| EncryptedMnemonicError::FailedToCreateRandomValue)?;

        let nonce_sequence = MnemonicNonceSequence::new()?;
        let nonce = nonce_sequence.get_current();

        let unbound_key = UnboundKey::new(&AES_256_GCM, key.as_bytes())
            .map_err(|_| EncryptedMnemonicError::FailedToCreateUnboundKey)?;
        let mut sealing_key = ring::aead::SealingKey::new(unbound_key, nonce_sequence);

        sealing_key.seal_in_place_append_tag(Aad::empty(), &mut mnemonic)
            .map_err(|_| EncryptedMnemonicError::FailedToEncryptMnemonic)?;

        //TODO: Investigate if the unbound key and sealing key gets overwritten when going out of scope.
        key.zeroize();

        Ok(Self {
            cypher_text: mnemonic,
            salt: salt,
            nonce_bytes: nonce,
        })
    }

    pub fn decrypt_mnemonic(&self, password:&Password) -> Result<Mnemonic, EncryptedMnemonicError> {
        let mut encryption_key = password.derive_mnemonic_encryption_key_from_salt(&self.salt);
        let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key.as_bytes())
            .map_err(|_| EncryptedMnemonicError::FailedToCreateUnboundKey)?;
        let nonce_sequence = MnemonicNonceSequence::with_nonce(&Nonce::assume_unique_for_key(self.nonce_bytes.clone()));
        let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

        let mut data = self.cypher_text.clone();

        let phrase = opening_key.open_in_place(Aad::empty(), &mut data)
            .map_err(|_| EncryptedMnemonicError::FailedToDecryptMnemonic)?;

        let mut phrase = String::from_utf8(phrase.to_vec())
            .map_err(|_| EncryptedMnemonicError::FailedToParseInvalidUtf8)?;

        //The validity check will have to be passed when the mnemonic first was passed to create the encrypted mnemonic, 
        //so the decrypted phrase will always be a valid mnemonic.
        let mnemonic = Mnemonic::from_phrase(&phrase, bip39::Language::English)
            .map_err(|_| EncryptedMnemonicError::FailedToConstructMnemonic)?;
        
        //Zeroize the plaintext mnemonic and encryption key before dropping it
        //Todo: Investigate zeroizing of the unbound and opening keys
        encryption_key.zeroize();
        data.zeroize();
        phrase.zeroize();

        Ok(mnemonic)
    }


    fn derive_encryption_key(password: &str, salt: &[u8;32]) -> [u8; 32] {
        let mut encryption_key: [u8;32] = [0;32];
        let iterations = unsafe {NonZeroU32::new_unchecked(ITERATIONS)};
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            salt.as_ref(),
            password.as_bytes(),
            encryption_key.as_mut_slice(),
        );
        encryption_key
    }

    fn create_salt() -> Result<[u8; 32], EncryptedMnemonicError> {
        let mut salt:[u8;32] = [0;32];
        SystemRandom::new().fill(&mut salt).map_err(|_| EncryptedMnemonicError::FailedToCreateRandomValue)?;
        
        Ok(salt)
    }
}

#[cfg(windows)]
use windows::{
    Win32::Security::{
        Credentials::{CredDeleteW, CredReadW, CredWriteW, CredFree},
        Credentials::{CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC}
    },
    core::{PCWSTR, PWSTR}
};

#[cfg(windows)]
impl EncryptedMnemonic {



    //Stores the mnemonic using the windows credentials manager 
    pub fn save_to_store(self, target_name: &str) -> Result<(), EncryptedMnemonicError> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();
        let mut blob = serde_json::to_vec(&self)
        .map_err(|_| EncryptedMnemonicError::FailedToParseEncryptedMnemonic)?;
    
        unsafe {
            let credentials = CREDENTIALW {
                Type: CRED_TYPE_GENERIC,
                TargetName: PWSTR(target_name.as_mut_ptr()),
                CredentialBlob: blob.as_mut_ptr(),
                CredentialBlobSize: (blob.len() * 2) as u32,
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                ..Default::default()
            };
            
            CredWriteW(&credentials, 0)
            .map_err(|err| EncryptedMnemonicError::FailedToSaveCredentials(Box::new(err)))?;
        }

        target_name.zeroize();
        blob.zeroize();

        Ok(())
    }

    ///Retrieves the EncryptedMnemonic from the Windows Credential Store.
    pub fn from_store(target_name: &str) -> Result<Self, EncryptedMnemonicError> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();
        let cred_ptr = std::ptr::null_mut();

        let result:Result<Self, EncryptedMnemonicError>;

        unsafe {
            match CredReadW(PCWSTR(target_name.as_mut_ptr()), CRED_TYPE_GENERIC, 0, cred_ptr) {
                Ok(_) => {
                    match cred_ptr.is_null() {
                        false => {
                            let cred = &**cred_ptr;
                            let serialized = std::slice::from_raw_parts(cred.CredentialBlob, cred.CredentialBlobSize as usize);
                            let encryptedmnemonic = serde_json::from_slice::<EncryptedMnemonic>(serialized)
                                .map_err(|_| EncryptedMnemonicError::FailedToParseEncryptedMnemonic)?;
                            result =  Ok(encryptedmnemonic)
                        }
                        true => {
                            result = Err(EncryptedMnemonicError::FailedToRetrieveCredentials(Box::new(windows::core::Error::OK)))
                        }
                    }
                }
                Err(err) => {                    
                    result = Err(EncryptedMnemonicError::FailedToRetrieveCredentials(Box::new(err)))
                }
            }
            if !cred_ptr.is_null() {
                CredFree(cred_ptr as *mut _)
            }
        }

        target_name.zeroize();

        result
    }

    pub fn delete_from_store(target_name: &str) -> Result<(), EncryptedMnemonicError> {
        let mut target_name = target_name.encode_utf16().collect::<Vec<u16>>();

        let result: Result<(), EncryptedMnemonicError>;
        unsafe {
            match CredDeleteW(PCWSTR(target_name.as_mut_ptr()), CRED_TYPE_GENERIC, 0) {
                Ok(_) => result = Ok(()),
                Err(err) => result = Err(EncryptedMnemonicError::FailedToDeleteCredentials(Box::new(err))),
            }
        }

        target_name.zeroize();

        result
    }
}





#[cfg(test)]
mod test {
    use super::*;

    impl EncryptedMnemonic {
        pub fn get_cypher_text(&self) -> Vec<u8> {
            self.cypher_text.clone()
        }
    }

    #[test]
    fn test_create_encrypted_mnemonic() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let encrypted_mnemonic = EncryptedMnemonic::new(&mnemonic.clone(), &password)
            .unwrap_or_else(|err| panic!("{err}"));

        println!("{:?}\n{:?}",mnemonic, encrypted_mnemonic.get_cypher_text());

        assert_ne!(mnemonic.clone().into_phrase().as_bytes().to_vec(), encrypted_mnemonic.get_cypher_text()[0..mnemonic.phrase().len()]);
    }

    #[test]
    fn test_decrypting_mnemonic() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let encrypted_mnemonic = EncryptedMnemonic::new(&mnemonic.clone(), &password)
            .unwrap_or_else(|err| panic!("{err}"));

        println!("{} \n{:?}", mnemonic.phrase(), encrypted_mnemonic.cypher_text);

        let decrypted = encrypted_mnemonic.decrypt_mnemonic(&password)
            .unwrap_or_else(|err| panic!("{err}"));

        assert_eq!(mnemonic.phrase(), decrypted.phrase())
    }

    #[cfg(windows)]
    #[test]
    fn test_save_read_delete_encrypted_mnemonic() {
        let mnemonic = Mnemonic::new(bip39::MnemonicType::Words24, bip39::Language::English);
        let password = Password::from("password99");
        let encrypted_mnemonic = EncryptedMnemonic::new(&mnemonic.clone(), &password).expect("Failed to create encrypted mnemonic");
        let target_name = "Ravault mnemonic store test";

        encrypted_mnemonic.save_to_store(target_name).expect("Failed to save encrypted mnemonic");
        let encrypted_mnemonic = EncryptedMnemonic::from_store(target_name).expect("Failed to retrieve encrypted mnemonic");
        let decrypted = encrypted_mnemonic.decrypt_mnemonic(&password).expect("Failed to decrypt mnemonoic");
        assert_eq!(decrypted.phrase(), mnemonic.phrase());

        EncryptedMnemonic::delete_from_store(target_name).expect("Failed to delete encrypted mnemonic from store");
        let result = EncryptedMnemonic::from_store(target_name);
        assert!(result.is_err())
    }
}


