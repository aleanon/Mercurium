pub mod ledger_reader;
mod port;
mod radix_dlt;
pub mod transaction_gateway;
// mod radix_official_gateway;

pub use ledger_reader::{LedgerReader, LedgerReaderError};
pub use transaction_gateway::{
    RadixGateway, SubmittedStatus, TransactionGateway, TransactionGatewayError,
};

use types::Network;

/// Composition root: construct the concrete ledger adapters for a network. As more adapters are
/// migrated behind their ports, they are wired here and injected into the wallet, so the wallet
/// depends only on the port traits.
pub fn radix_transaction_gateway(network: Network) -> RadixGateway {
    RadixGateway::new(network)
}
