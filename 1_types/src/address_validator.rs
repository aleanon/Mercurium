use std::{borrow::Cow, fmt::Debug};

use once_cell::sync::Lazy;
use scrypto::{address::AddressBech32Decoder, network::NetworkDefinition, types::EntityType};

use crate::Network;

pub static MAINNET_DECODER: Lazy<AddressBech32Decoder> =
    Lazy::new(|| AddressBech32Decoder::new(&NetworkDefinition::mainnet()));

pub static STOKENET_DECODER: Lazy<AddressBech32Decoder> =
    Lazy::new(|| AddressBech32Decoder::new(&NetworkDefinition::stokenet()));

pub struct AddressValidator;

impl AddressValidator {
    pub fn is_valid_address(
        network: Network,
        expected_entity_type: EntityType,
        address: &str,
    ) -> bool {
        match network {
            Network::Mainnet => {
                let Ok((entity_type, _)) = MAINNET_DECODER.validate_and_decode(address) else {
                    return false;
                };
                entity_type == expected_entity_type
            }
            Network::Stokenet => {
                let Ok((entity_type, _)) = STOKENET_DECODER.validate_and_decode(address) else {
                    return false;
                };
                entity_type == expected_entity_type
            }
        }
    }
}
