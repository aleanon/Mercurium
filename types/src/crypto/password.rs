use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::{encryption_error::EncryptionError, key::Key, salt::Salt};

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Failed to derive Encryption key: {0}")]
    EncryptionKeyError(#[from] EncryptionError),
}

///A secure wrapper around a ``String`` that implements ``ZeroizeOnDrop`` to make sure the password is cleaned from memory before it is dropped.
///It allocates with the max password size once, to make sure the ``zeroize`` method works, as ``Zeroize`` can't guarantee data is properly removed
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
            self.push(c)
        }
        //Else do nothing
    }

    pub fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }

    pub fn push_str(&mut self, s: &str) {
        match self.0.len() {
            len if len < Self::MAX_LEN => {
                let max_len = Self::MAX_LEN - len;
                match s.len() {
                    s_len if s_len <= max_len => {
                        self.0.push_str(s);
                    }
                    _ => {
                        self.0.push_str(s[..max_len].trim());
                    }
                }
            }
            _ => { /*Do nothing*/ }
        }
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn derive_new_db_encryption_key(&self) -> Result<(Key, Salt), PasswordError> {
        let salt = Salt::new()?;
        let key = Key::db_encryption_key(&salt, &self);

        Ok((key, salt))
    }

    pub fn derive_db_encryption_key_from_salt(&self, salt: &Salt) -> Key {
        Key::db_encryption_key(salt, &self)
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
            len if len > 1 && len <= Self::MAX_LEN => string.push_str(
                value
                    .get(0..len)
                    .expect("Failed to copy password, invalid index"),
            ),
            0 => {}
            _ => string.push_str(
                value
                    .get(0..Self::MAX_LEN)
                    .expect("Failed to copy password, invalid index"),
            ),
        };

        Self(string)
    }
}

impl From<String> for Password {
    fn from(mut value: String) -> Self {
        let mut string = String::with_capacity(Self::MAX_LEN);
        match value.len() {
            0 => {}
            len if len > 1 && len <= Self::MAX_LEN => {
                string.push_str(
                    value
                        .get(0..len)
                        .expect("Failed to copy password, invalid index"),
                );
                value.zeroize()
            }
            _ => {
                string.push_str(
                    value
                        .get(0..Self::MAX_LEN)
                        .expect("Failed to copy password, invalid index"),
                );
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
