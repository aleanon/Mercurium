use std::str::FromStr;

use crate::{Network, ParseAddrError};

pub struct ComponentAddress(Network, [u8; Self::ADDRESS_LENGTH]);

impl ComponentAddress {
    const PREFIX: [u8; Self::PREFIX_LENGTH] =
        [b'c', b'o', b'm', b'p', b'o', b'n', b'e', b'n', b't', b'_'];
    const PREFIX_LENGTH: usize = 10;
    const ADDRESS_LENGTH: usize = 54;
    const PREFIX_STR: &'static str = "component_";

    fn as_str(&self) -> &str {
        match self.0 {
            Network::Mainnet => unsafe {
                let slice = std::slice::from_raw_parts(
                    [
                        Self::PREFIX.as_slice(),
                        Network::MAINNET_PREFIX.as_slice(),
                        self.1.as_slice(),
                    ]
                    .as_ptr() as *const u8,
                    Self::PREFIX_LENGTH + Network::MAINNET_PREFIX_LENGTH + Self::ADDRESS_LENGTH,
                );
                std::str::from_utf8_unchecked(slice)
            },
            Network::Stokenet => unsafe {
                let slice = std::slice::from_raw_parts(
                    [
                        Self::PREFIX.as_slice(),
                        Network::STOKENET_PREFIX.as_slice(),
                        self.1.as_slice(),
                    ]
                    .as_ptr() as *const u8,
                    Self::PREFIX_LENGTH + Network::STOKENET_PREFIX_LENGTH + Self::ADDRESS_LENGTH,
                );
                std::str::from_utf8_unchecked(slice)
            },
        }
    }

    pub fn as_str2(&self) -> &str {
        let network_prefix = self.0.prefix().as_bytes();
        unsafe {
            let slice = std::slice::from_raw_parts(
                [
                    Self::PREFIX_STR.as_bytes(),
                    network_prefix,
                    self.1.as_slice(),
                ]
                .as_ptr() as *const u8,
                Self::PREFIX_STR.len() + network_prefix.len() + Self::ADDRESS_LENGTH,
            );
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl FromStr for ComponentAddress {
    type Err = ParseAddrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_ascii() {
            let address_start = s.len() - Self::ADDRESS_LENGTH;
            let address = &s[address_start..];
            let prefix = &s[..Self::PREFIX_LENGTH];
            if prefix != Self::PREFIX_STR {
                return Err(ParseAddrError::InvalidPrefix);
            }
            let network_prefix = &s[Self::PREFIX_LENGTH..address_start];
            let network = match network_prefix {
                Network::MAINNET_PREFIX_STR => Network::Mainnet,
                Network::STOKENET_PREFIX_STR => Network::Stokenet,
                _ => return Err(ParseAddrError::InvalidPrefix),
            };
            Ok(Self(network, address.as_bytes().try_into()?))
        } else {
            Err(ParseAddrError::NonAsciiCharacter)
        }
    }
}
