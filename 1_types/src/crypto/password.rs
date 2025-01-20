use std::{fmt::Debug, num::NonZeroU32};

use async_sqlite::rusqlite::{self, types::FromSql, ToSql};
use ring::pbkdf2::{self, PBKDF2_HMAC_SHA512};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::debug_info;
use crate::unwrap_unreachable::UnwrapUnreachable;

use super::{encryption_error::CryptoError, key::Key, salt::Salt};

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Failed to derive Encryption key: {0}")]
    EncryptionKeyError(#[from] CryptoError),
}

///A secure wrapper around a `String` that implements `ZeroizeOnDrop` to make sure the password is cleaned from memory before it is dropped.
///It allocates with the max password size once, to make sure the `zeroize` method works. `Zeroize` can not guarantee data is properly removed
///if the memory has been reallocated.
#[derive(Clone, ZeroizeOnDrop, PartialEq)]
pub struct Password(String);

impl Password {
    pub const MAX_LEN: usize = 64;
    pub const MIN_LEN: usize = 16;

    pub fn new() -> Self {
        Self(String::with_capacity(Self::MAX_LEN))
    }

    pub fn push(&mut self, c: char) {
        if self.0.len() < Self::MAX_LEN {
            self.0.push(c)
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }

    /// Replaces the current password with the supplied [&str]
    pub fn replace(&mut self, s: &str) {
        self.0.clear();
        self.push_str(s);
    }

    pub fn push_str(&mut self, s: &str) {
        if self.len() < Self::MAX_LEN {
            let len_left = Self::MAX_LEN - self.len();
            if s.len() <= len_left {
                self.0.push_str(s);
            } else {
                self.0.push_str(&s[..len_left]);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    // pub fn derive_new_db_encryption_key(&self) -> Result<(Key<DataBaseKey>, Salt), PasswordError> {
    //     let salt = Salt::new()?;
    //     let key = Key::new(&self.as_str(), &salt);

    //     Ok((key, salt))
    // }

    // pub fn derive_db_encryption_key_from_salt(&self, salt: &Salt) -> Key<DataBaseKey> {
    //     Key::new(&self.as_str(), salt)
    // }

    pub fn derive_db_encryption_key_hash_from_salt(&self, salt: &Salt) -> HashedPassword {
        HashedPassword::new(salt, self)
    }

    // pub fn derive_new_mnemonic_encryption_key(&self) -> Result<(Key<MnemonicKey>, Salt), PasswordError> {
    //     let salt = Salt::new()?;
    //     let key = Key::new(&self.as_str(), &salt);

    //     Ok((key, salt))
    // }

    // pub fn derive_mnemonic_encryption_key_from_salt(&self, salt: &Salt) -> Key<MnemonicKey> {
    //     Key::new(&self.as_str(), salt)
    // }
}

impl Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Password length: {}", self.0.len())
    }
}

impl Default for Password {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for Password {
    fn from(value: &str) -> Self {
        let mut string = String::with_capacity(Self::MAX_LEN);
        match value.len() {
            len @ 1..=Self::MAX_LEN => string.push_str(&value[..len]),
            0 => {}
            _ => string.push_str(&value[..Self::MAX_LEN]),
        };

        Self(string)
    }
}

impl From<String> for Password {
    fn from(mut value: String) -> Self {
        let mut string = String::with_capacity(Self::MAX_LEN);
        match value.len() {
            len @ 1..=Self::MAX_LEN => string.push_str(&value[..len]),
            0 => {}
            _ => string.push_str(&value[..Self::MAX_LEN]),
        }
        value.zeroize();
        Self(string)
    }
}

#[derive(PartialEq, Eq, ZeroizeOnDrop)]
pub struct HashedPassword([u8; Self::LENGTH]);

impl HashedPassword {
    const LENGTH: usize = 64;
    const HASH_ITERATIONS: u32 = 50000;

    pub fn new(salt: &Salt, password: &Password) -> Self {
        let mut hash = [0u8; Self::LENGTH];
        let iterations = NonZeroU32::new(Self::HASH_ITERATIONS).unwrap_unreachable(debug_info!(
            "Zero value supplied for password hash iterations"
        ));

        pbkdf2::derive(
            PBKDF2_HMAC_SHA512,
            iterations,
            salt.as_bytes(),
            password.as_str().as_bytes(),
            &mut hash,
        );

        Self(hash)
    }
}

impl ToSql for HashedPassword {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Blob(&self.0),
        ))
    }
}

impl FromSql for HashedPassword {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let blob = value.as_blob()?;
        Ok(Self(blob.try_into().map_err(|_| {
            rusqlite::types::FromSqlError::InvalidBlobSize {
                expected_size: Self::LENGTH,
                blob_size: blob.len(),
            }
        })?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_from_str() {
        let phrase = "password99";
        let password = Password::from(phrase);
        assert_eq!(password, Password(phrase.to_string()));

        let phrase = "";
        let password = Password::from(phrase);
        assert_eq!(password, Password(phrase.to_string()));

        let phrase = "p";
        let password = Password::from(phrase);
        assert_eq!(password, Password(phrase.to_string()));

        let phrase =
            "tolongpasswordthatshouldbecutoffbeforethefullpasswordiscopiedintothepasswordtype";
        let target = "tolongpasswordthatshouldbecutoffbeforethefullpasswordiscopiedint";
        let password = Password::from(phrase);
        assert_eq!(password, Password(target.to_string()));
    }

    #[test]
    fn test_password_push() {
        let phrase = "password99";
        let mut password = Password(phrase.to_string());
        password.push('a');

        assert_eq!(password, Password("password99a".to_string()));

        let phrase = "tolongpasswordthatshouldbecutoffbeforethefullpasswordiscopiedint";
        let mut password = Password(phrase.to_string());
        password.push('a');

        assert_eq!(password, Password(phrase.to_string()))
    }

    #[test]
    fn test_password_push_str() {
        let mut password = Password::new();
        password.push_str("");
        assert_eq!(password, Password("".to_string()));

        let mut password = Password::new();
        password.push_str("1");
        assert_eq!(password, Password("1".to_string()));

        let phrase = "password99";
        let mut password = Password(phrase.to_string());
        password.push_str("password98");
        assert_eq!(password, Password("password99password98".to_string()));

        let phrase = "tolongpasswordthatshouldbecutoffbeforethefullpasswordiscopiedint";
        let mut password = Password(phrase.to_string());
        password.push_str("shouldbediscarded");
        assert_eq!(password, Password(phrase.to_string()));

        let mut password = Password::new();
        password.push_str(
            "tolongpasswordthatshouldbecutoffbeforethefullpasswordiscopiedinthshouldbecut",
        );
        assert_eq!(
            password,
            Password(
                "tolongpasswordthatshouldbecutoffbeforethefullpasswordiscopiedint".to_string()
            )
        );

        let mut password =
            Password("tolongpasswordthatshouldbecutoffbeforethefullpassword".to_string());
        password.push_str("iscopiedintshouldbecut");
        assert_eq!(
            password,
            Password(
                "tolongpasswordthatshouldbecutoffbeforethefullpasswordiscopiedint".to_string()
            )
        );
    }

    #[test]
    fn test_capacity_does_not_change() {
        let mut password = Password::new();
        let capacity_before_change = password.0.capacity();
        assert_eq!(capacity_before_change, Password::MAX_LEN);

        let to_long_password =
            "tolongpasswordthatshouldbecutoffbeforethefullpasswordiscopiedinthshouldbecut";
        password.push_str(&to_long_password);

        assert_eq!(password.len(), Password::MAX_LEN);
        assert_eq!(capacity_before_change, password.0.capacity())
    }
}
