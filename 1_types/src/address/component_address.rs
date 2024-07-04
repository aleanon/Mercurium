use std::str::FromStr;

use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable, Network, ParseAddrError};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentAddress {
    Mainnet([u8; Self::MAINNET_ADDRESS_LENGTH]),
    Stokenet([u8; Self::STOKENET_ADDRESS_LENGTH]),
}

impl ComponentAddress {
    const MAINNET_ADDRESS_LENGTH: usize =
        Self::PREFIX_LENGTH + Self::ADDRESS_LENGTH + Network::MAINNET_PREFIX_LENGTH;

    const STOKENET_ADDRESS_LENGTH: usize =
        Self::PREFIX_LENGTH + Self::ADDRESS_LENGTH + Network::STOKENET_PREFIX_LENGTH;

    const MAINNET_NETWORK_PREFIX_END: usize = Self::PREFIX_LENGTH + Network::MAINNET_PREFIX_LENGTH;

    const STOKENET_NETWORK_PREFIX_END: usize =
        Self::PREFIX_LENGTH + Network::STOKENET_PREFIX_LENGTH;

    const PREFIX: &'static str = "component_";

    const PREFIX_LENGTH: usize = 10;
    const ADDRESS_LENGTH: usize = 54;

    const MAINNET_CHECKSUM_START_INDEX: usize =
        Self::MAINNET_ADDRESS_LENGTH - Self::CHECKSUM_LENGTH;

    const STOKENET_CHECKSUM_START_INDEX: usize =
        Self::STOKENET_ADDRESS_LENGTH - Self::CHECKSUM_LENGTH;

    const CHECKSUM_LENGTH: usize = 6;

    pub fn as_str(&self) -> &str {
        match self {
            Self::Mainnet(slice) => std::str::from_utf8(slice)
                .unwrap_unreachable(debug_info!("Invalid utf8 in component address")),
            Self::Stokenet(slice) => std::str::from_utf8(slice)
                .unwrap_unreachable(debug_info!("Invalid utf8 in component address")),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Mainnet(slice) => slice,
            Self::Stokenet(slice) => slice,
        }
    }

    pub fn checksum(&self) -> [u8; Self::CHECKSUM_LENGTH] {
        match self {
            Self::Mainnet(slice) => slice[Self::MAINNET_CHECKSUM_START_INDEX..]
                .try_into()
                .unwrap_unreachable(debug_info!("Invalid checksum length")),
            Self::Stokenet(slice) => slice[Self::STOKENET_CHECKSUM_START_INDEX..]
                .try_into()
                .unwrap_unreachable(debug_info!("Invalid checksum length")),
        }
    }

    pub fn checksum_str(&self) -> &str {
        match self {
            Self::Mainnet(slice) => {
                std::str::from_utf8(&slice[Self::MAINNET_CHECKSUM_START_INDEX..])
                    .unwrap_unreachable(debug_info!("Invalid UTF-8 in AccountAddress"))
            }
            Self::Stokenet(slice) => {
                std::str::from_utf8(&slice[Self::STOKENET_CHECKSUM_START_INDEX..])
                    .unwrap_unreachable(debug_info!("Invalid UTF-8 in AccountAddress"))
            }
        }
    }
}

impl AsRef<[u8]> for ComponentAddress {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Mainnet(slice) => slice.as_slice(),
            Self::Stokenet(slice) => slice.as_slice(),
        }
    }
}

impl ToString for ComponentAddress {
    fn to_string(&self) -> String {
        match self {
            Self::Mainnet(array) => String::from_utf8(array.as_slice().to_vec())
                .unwrap_unreachable(debug_info!("Invalid utf8 in component address")),
            Self::Stokenet(array) => String::from_utf8(array.as_slice().to_vec())
                .unwrap_unreachable(debug_info!("Invalid utf8 in component address")),
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
            if prefix != Self::PREFIX {
                return Err(ParseAddrError::InvalidPrefix);
            }
            let network_prefix = &s[Self::PREFIX_LENGTH..address_start];
            let component_address = match network_prefix {
                Network::MAINNET_PREFIX_STR => {
                    let mut address_array = [0u8; Self::MAINNET_ADDRESS_LENGTH];
                    address_array[0..Self::PREFIX_LENGTH].copy_from_slice(prefix.as_bytes());
                    address_array[Self::PREFIX_LENGTH..Self::MAINNET_NETWORK_PREFIX_END]
                        .copy_from_slice(Network::MAINNET_PREFIX_STR.as_bytes());
                    address_array[Self::MAINNET_NETWORK_PREFIX_END..]
                        .copy_from_slice(address.as_bytes());
                    Self::Mainnet(address_array)
                }
                Network::STOKENET_PREFIX_STR => {
                    let mut address_array = [0u8; Self::STOKENET_ADDRESS_LENGTH];
                    address_array[0..Self::PREFIX_LENGTH].copy_from_slice(prefix.as_bytes());
                    address_array[Self::PREFIX_LENGTH..Self::STOKENET_NETWORK_PREFIX_END]
                        .copy_from_slice(Network::STOKENET_PREFIX_STR.as_bytes());
                    address_array[Self::STOKENET_NETWORK_PREFIX_END..]
                        .copy_from_slice(address.as_bytes());
                    Self::Stokenet(address_array)
                }
                _ => return Err(ParseAddrError::InvalidPrefix),
            };
            Ok(component_address)
        } else {
            Err(ParseAddrError::NonAsciiCharacter)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_component_address_to_string() {
        let address = "component_rdx16y60m8p2lxl72rdqcxh6wj270ckku7e3hrr6fra05f9p34zlqwgd0k";

        let component_address = ComponentAddress::from_str(address).unwrap();
        assert_eq!(
            component_address,
            ComponentAddress::Mainnet(address.as_bytes().try_into().unwrap())
        );

        assert_eq!(component_address.to_string(), address.to_string());
    }
}
