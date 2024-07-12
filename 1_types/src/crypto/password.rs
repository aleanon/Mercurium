use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::hashed_password::HashedPassword;

use super::{encryption_error::EncryptionError, key::Key, salt::Salt, HexKey};

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Failed to derive Encryption key: {0}")]
    EncryptionKeyError(#[from] EncryptionError),
}

///A secure wrapper around a ``String`` that implements ``ZeroizeOnDrop`` to make sure the password is cleaned from memory before it is dropped.
///It allocates with the max password size once, to make sure the ``zeroize`` method works. ``Zeroize`` can not guarantee data is properly removed
///if the memory has been reallocated.
#[derive(Debug, Clone, ZeroizeOnDrop, PartialEq)]
pub struct Password(String);

impl Password {
    pub const MAX_LEN: usize = 64;

    pub fn new() -> Self {
        Self(String::with_capacity(Self::MAX_LEN))
    }

    pub fn push(&mut self, c: char) {
        if self.0.len() < Self::MAX_LEN {
            self.0.push(c)
        }
        //Else do nothing
    }

    pub fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }

    pub fn push_str(&mut self, s: &str) {
        let len = self.0.len();
        if len < Self::MAX_LEN {
            let max_len = Self::MAX_LEN - len;
            if s.len() <= max_len {
                self.0.push_str(s);
            } else {
                self.0.push_str(&s[..max_len]);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn derive_new_db_encryption_key(&self) -> Result<(HexKey, Salt), PasswordError> {
        let salt = Salt::new()?;
        let key = Key::db_encryption_key(&salt, &self);

        Ok((key.into_hex_key(), salt))
    }

    pub fn derive_db_encryption_key_from_salt(&self, salt: &Salt) -> HexKey {
        Key::db_encryption_key(salt, &self).into_hex_key()
    }

    pub fn derive_db_encryption_key_hash_from_salt(&self, salt: &Salt) -> HashedPassword {
        HashedPassword::db_key_hash(salt, self)
    }

    pub fn derive_new_mnemonic_encryption_key(&self) -> Result<(Key, Salt), PasswordError> {
        let salt = Salt::new()?;
        let key = Key::mnemonic_encryption_key(&salt, &self);

        Ok((key, salt))
    }

    pub fn derive_mnemonic_encryption_key_from_salt(&self, salt: &Salt) -> Key {
        Key::mnemonic_encryption_key(salt, &self)
    }
}

impl From<&str> for Password {
    fn from(value: &str) -> Self {
        let mut string = String::with_capacity(Self::MAX_LEN);
        match value.len() {
            len if len > 0 && len <= Self::MAX_LEN => string.push_str(&value[0..len]),
            0 => {}
            _ => string.push_str(&value[0..Self::MAX_LEN]),
        };

        Self(string)
    }
}

impl From<String> for Password {
    fn from(mut value: String) -> Self {
        let mut string = String::with_capacity(Self::MAX_LEN);
        match value.len() {
            len if len > 0 && len <= Self::MAX_LEN => {
                string.push_str(&value[0..len]);
                value.zeroize()
            }
            0 => {}
            _ => {
                string.push_str(&value[0..Self::MAX_LEN]);
                value.zeroize()
            }
        }
        Self(string)
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
}
