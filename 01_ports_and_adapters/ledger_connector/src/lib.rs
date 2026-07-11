#![forbid(unsafe_code)]

pub mod ledger_reader;
mod port;
mod radix_dlt;
pub mod transaction_gateway;

pub use ledger_reader::{LedgerReader, LedgerReaderError};
pub use transaction_gateway::{
    RadixGateway, SubmittedStatus, TransactionGateway, TransactionGatewayError,
};

use std::sync::Arc;
use types::Network;

/// Composition root: construct the concrete ledger adapters for a network. As more adapters are
/// migrated behind their ports, they are wired here and injected into the wallet, so the wallet
/// depends only on the port traits.
pub fn radix_transaction_gateway(network: Network) -> RadixGateway {
    RadixGateway::new(network)
}

/// Produces a [`TransactionGateway`] for a given network.
///
/// The wallet runs two networks and can switch between them, and the network for an operation is
/// determined by the account/transaction (not a single global), so the injected capability is a
/// *provider keyed by network*, never one pinned gateway instance. Production wires
/// [`RadixGatewayProvider`]; tests wire a fake (see the `testing` module).
pub trait GatewayProvider: Send + Sync {
    fn gateway(&self, network: Network) -> Arc<dyn TransactionGateway + Send + Sync>;
}

/// The production provider, backed by the Radix Gateway API.
pub struct RadixGatewayProvider;

impl GatewayProvider for RadixGatewayProvider {
    fn gateway(&self, network: Network) -> Arc<dyn TransactionGateway + Send + Sync> {
        Arc::new(RadixGateway::new(network))
    }
}

/// In-memory test doubles for headless verification (enabled by the `testing` feature, or under
/// `cfg(test)` within this crate).
#[cfg(any(test, feature = "testing"))]
pub mod testing {
    use super::*;
    use async_trait::async_trait;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// A deterministic, offline [`TransactionGateway`].
    ///
    /// - `current_epoch` returns the configured `epoch`.
    /// - `submit` records the notarized hex it was given (inspectable via [`FakeGateway::submissions`])
    ///   and reports it as non-duplicate.
    /// - `status` pops the next queued status, repeating the last one once the queue drains, so a
    ///   `[Pending, CommittedSuccess]` queue lets `poll_until_settled` terminate without waiting.
    pub struct FakeGateway {
        pub epoch: u64,
        submitted: Mutex<Vec<String>>,
        statuses: Mutex<VecDeque<SubmittedStatus>>,
    }

    impl FakeGateway {
        /// A gateway that immediately reports `CommittedSuccess`.
        pub fn committed() -> Self {
            Self::new(1000, [SubmittedStatus::CommittedSuccess])
        }

        pub fn new(
            epoch: u64,
            statuses: impl IntoIterator<Item = SubmittedStatus>,
        ) -> Self {
            Self {
                epoch,
                submitted: Mutex::new(Vec::new()),
                statuses: Mutex::new(statuses.into_iter().collect()),
            }
        }

        /// The notarized transactions submitted to this gateway, in order.
        pub fn submissions(&self) -> Vec<String> {
            self.submitted.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl TransactionGateway for FakeGateway {
        async fn current_epoch(&self) -> Result<u64, TransactionGatewayError> {
            Ok(self.epoch)
        }

        async fn submit(&self, notarized_hex: &str) -> Result<bool, TransactionGatewayError> {
            self.submitted.lock().unwrap().push(notarized_hex.to_string());
            Ok(false)
        }

        async fn status(
            &self,
            _intent_hash_id: &str,
        ) -> Result<SubmittedStatus, TransactionGatewayError> {
            let mut q = self.statuses.lock().unwrap();
            Ok(match q.len() {
                0 => SubmittedStatus::Unknown,
                1 => *q.front().unwrap(),
                _ => q.pop_front().unwrap(),
            })
        }
    }

    /// A [`GatewayProvider`] that hands out a shared [`FakeGateway`] regardless of network.
    pub struct FakeGatewayProvider {
        gateway: Arc<FakeGateway>,
    }

    impl FakeGatewayProvider {
        pub fn new(gateway: FakeGateway) -> Self {
            Self { gateway: Arc::new(gateway) }
        }

        /// The shared fake, e.g. to assert on its recorded submissions.
        pub fn gateway_ref(&self) -> Arc<FakeGateway> {
            self.gateway.clone()
        }
    }

    impl GatewayProvider for FakeGatewayProvider {
        fn gateway(&self, _network: Network) -> Arc<dyn TransactionGateway + Send + Sync> {
            self.gateway.clone()
        }
    }
}
