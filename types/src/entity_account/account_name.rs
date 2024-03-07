use std::str::FromStr;



const ACC_NAME_LENGTH: usize = 60;


pub enum AccNameError {
    NonAsciiCharacter,
    InvalidLength {
        expected: usize,
        found: usize,
    },
}


pub struct AccountName([u8;ACC_NAME_LENGTH]);


impl AccountName {
    pub fn from_slice(slice: &[u8]) -> Result<Self, AccNameError> {
        if !slice.is_ascii() {
            return Err(AccNameError::NonAsciiCharacter)
        }
        Ok(Self(slice.try_into().map_err(|_| AccNameError::InvalidLength { expected: ACC_NAME_LENGTH, found: slice.len()})?))
    }
}

impl FromStr for AccountName {
    type Err = AccNameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(AccNameError::NonAsciiCharacter)
        }
        Ok(Self(s.as_bytes().try_into().map_err(|_| AccNameError::InvalidLength { expected: ACC_NAME_LENGTH, found: s.len()})?))
    }
}