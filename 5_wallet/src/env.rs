//! `Env` — the injected capability bundle.
//!
//! Holds the *how to do I/O* dependencies (read-only, `&self`-method trait objects), shared as
//! `Arc` so they can be cloned into `'static` async tasks. Mutable application state (the open
//! database, `Profile`, `Settings`, login attempts) is **not** here — it lives in `App` / the
//! `Wallet` typestate. See `.ai_docs/di_testability_plan.md` §1a.
//!
//! Fields are added as their seams land: `secrets` (phase 2) and `gateways` (phase 1/2.5) today;
//! `paths`, the data store, and the profile/settings stores follow in phases 3–4.

use std::sync::Arc;

use ledger_connector::{GatewayProvider, RadixGatewayProvider};
use secrets_store::SecretsStore;
use types::AppPathInner;

#[derive(Clone)]
pub struct Env {
    /// On-disk path set (no global). Production = platform default; tests = a temp-dir root.
    pub paths: Arc<AppPathInner>,
    /// Secrets store: OS keychain in production, in-memory under test.
    pub secrets: Arc<dyn SecretsStore>,
    /// Per-network transaction-gateway provider (never a single pinned gateway — see §1a).
    pub gateways: Arc<dyn GatewayProvider>,
}

impl Env {
    /// The production environment: platform paths + OS secrets + the Radix gateway.
    pub fn production() -> Self {
        let paths = Arc::new(
            AppPathInner::new().expect("unable to establish application directory"),
        );
        Self {
            secrets: secrets_store::production(paths.clone()),
            paths,
            gateways: Arc::new(RadixGatewayProvider),
        }
    }

    /// Inject specific capabilities (used by tests / `Preset` boot closures).
    pub fn new(
        paths: Arc<AppPathInner>,
        secrets: Arc<dyn SecretsStore>,
        gateways: Arc<dyn GatewayProvider>,
    ) -> Self {
        Self { paths, secrets, gateways }
    }

    /// The transaction gateway for `network` (convenience over `self.gateways`, so callers don't
    /// need the [`GatewayProvider`] trait in scope).
    pub fn gateway(
        &self,
        network: types::Network,
    ) -> Arc<dyn ledger_connector::TransactionGateway + Send + Sync> {
        self.gateways.gateway(network)
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::production()
    }
}

// Trait objects aren't `Debug`; keep `WalletData`'s derived `Debug` working without printing
// capability internals.
impl std::fmt::Debug for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Env").finish_non_exhaustive()
    }
}
