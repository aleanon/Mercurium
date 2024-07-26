use once_cell::sync::Lazy;
use scrypto::{address::AddressBech32Decoder, network::NetworkDefinition, types::EntityType};

use crate::Network;

pub static MAINNET_DECODER: Lazy<AddressBech32Decoder> =
    Lazy::new(|| AddressBech32Decoder::new(&NetworkDefinition::mainnet()));

pub static STOKENET_DECODER: Lazy<AddressBech32Decoder> =
    Lazy::new(|| AddressBech32Decoder::new(&NetworkDefinition::stokenet()));

pub struct AddressValidator;

impl AddressValidator {
    pub fn is_valid_account(network: Network, address: &str) -> bool {
        match network {
            Network::Mainnet => {
                let Ok((entity_type, _)) = MAINNET_DECODER.validate_and_decode(address) else {
                    return false;
                };
                match entity_type {
                    EntityType::GlobalVirtualEd25519Account
                    | EntityType::GlobalVirtualSecp256k1Account => true,
                    _ => false,
                }
            }
            Network::Stokenet => {
                let Ok((entity_type, _)) = STOKENET_DECODER.validate_and_decode(address) else {
                    return false;
                };
                match entity_type {
                    EntityType::GlobalVirtualEd25519Account
                    | EntityType::GlobalVirtualSecp256k1Account => true,
                    _ => false,
                }
            }
        }
    }

    pub fn is_valid_identity(network: Network, address: &str) -> bool {
        match network {
            Network::Mainnet => {
                let Ok((entity_type, _)) = MAINNET_DECODER.validate_and_decode(address) else {
                    return false;
                };
                match entity_type {
                    EntityType::GlobalVirtualEd25519Identity
                    | EntityType::GlobalVirtualSecp256k1Identity => true,
                    _ => false,
                }
            }
            Network::Stokenet => {
                let Ok((entity_type, _)) = STOKENET_DECODER.validate_and_decode(address) else {
                    return false;
                };
                match entity_type {
                    EntityType::GlobalVirtualEd25519Identity
                    | EntityType::GlobalVirtualSecp256k1Identity => true,
                    _ => false,
                }
            }
        }
    }
}
