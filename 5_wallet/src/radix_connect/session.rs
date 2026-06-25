//! Radix Connect session loop: receive an interaction, hand it to a handler, send the response.
//!
//! This ties the transport ([`RelayTransport`]) to the dispatch logic. It is generic over the
//! handler so the wallet can supply real auth/transaction handling while tests use a mock, and
//! generic over the transport so the live relay and an in-memory mock share the same driver.

use std::future::Future;

use super::{ConnectError, RelayTransport, WalletInteraction, WalletResponse};

/// Handles a single dApp interaction, producing the wallet's response (auth proof, submitted
/// transaction id, or a rejection).
pub trait InteractionHandler {
    fn handle(
        &self,
        interaction: WalletInteraction,
    ) -> impl Future<Output = WalletResponse> + Send;
}

/// Processes one interaction: `next_interaction` → `handle` → `send_response`.
pub async fn process_next<T, H>(transport: &mut T, handler: &H) -> Result<(), ConnectError>
where
    T: RelayTransport,
    H: InteractionHandler,
{
    let interaction = transport.next_interaction().await?;
    let interaction_id = interaction.interaction_id.clone();
    let response = handler.handle(interaction).await;
    transport.send_response(&interaction_id, &response).await
}

/// Runs the session loop until the transport returns an error (e.g. the session ends or the
/// relay connection drops).
pub async fn run_session<T, H>(mut transport: T, handler: H) -> Result<(), ConnectError>
where
    T: RelayTransport,
    H: InteractionHandler,
{
    loop {
        process_next(&mut transport, &handler).await?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::radix_connect::{DappMetadata, WalletRequest};
    use std::cell::RefCell;
    use types::Network;

    struct MockTransport {
        queued: Option<WalletInteraction>,
        sent: RefCell<Vec<(String, WalletResponse)>>,
    }

    impl RelayTransport for MockTransport {
        async fn next_interaction(&mut self) -> Result<WalletInteraction, ConnectError> {
            self.queued
                .take()
                .ok_or_else(|| ConnectError::Transport("no more interactions".to_string()))
        }

        async fn send_response(
            &mut self,
            interaction_id: &str,
            response: &WalletResponse,
        ) -> Result<(), ConnectError> {
            self.sent
                .borrow_mut()
                .push((interaction_id.to_string(), response.clone()));
            Ok(())
        }
    }

    struct RejectingHandler;

    impl InteractionHandler for RejectingHandler {
        async fn handle(&self, _interaction: WalletInteraction) -> WalletResponse {
            WalletResponse::Rejected {
                reason: "test".to_string(),
            }
        }
    }

    fn login_interaction() -> WalletInteraction {
        WalletInteraction {
            interaction_id: "abc-123".to_string(),
            metadata: DappMetadata {
                dapp_definition_address: "account_tdx_2_1".to_string(),
                origin: "https://dapp.example".to_string(),
                network: Network::Stokenet,
            },
            request: WalletRequest::AuthLogin { challenge: [0u8; 32] },
        }
    }

    #[test]
    fn process_next_dispatches_and_sends_response() {
        let runtime = deps::tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut transport = MockTransport {
                queued: Some(login_interaction()),
                sent: RefCell::new(Vec::new()),
            };

            process_next(&mut transport, &RejectingHandler)
                .await
                .expect("processes one interaction");

            let sent = transport.sent.borrow();
            assert_eq!(sent.len(), 1);
            assert_eq!(sent[0].0, "abc-123");
            assert!(matches!(sent[0].1, WalletResponse::Rejected { .. }));
        });
    }

    #[test]
    fn run_session_ends_when_transport_drains() {
        let runtime = deps::tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let transport = MockTransport {
                queued: Some(login_interaction()),
                sent: RefCell::new(Vec::new()),
            };
            // One interaction then the transport errors -> the loop returns that error.
            let result = run_session(transport, RejectingHandler).await;
            assert!(result.is_err());
        });
    }
}
