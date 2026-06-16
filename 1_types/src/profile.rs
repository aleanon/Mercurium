use deps::*;

use serde::{Deserialize, Serialize};

use crate::{Network, persona::PersonaData, theme::Theme};

/// The versioned wallet **Profile** — the single, serializable source of truth for the wallet's
/// configuration, mirroring the official wallet's profile model. It holds the parts that are
/// backed up (without the seed): gateways, preferences, authorized dApps, and factor-source
/// metadata. Accounts and personas continue to live in their own (encrypted) storage and are
/// referenced by address; a later revision can fold them in here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    /// Schema/snapshot version, bumped on breaking changes so older backups can be migrated.
    pub version: u32,
    pub gateways: Gateways,
    pub app_preferences: AppPreferences,
    pub authorized_dapps: Vec<AuthorizedDapp>,
    /// Metadata about the factor sources the user has added (never the secret material itself).
    pub factor_sources: Vec<FactorSourceMeta>,
    /// Security Shields (multi-factor structures) the user has defined; entities can be secured
    /// with one of these (creating an on-ledger Access Controller).
    pub security_structures: Vec<SecurityStructure>,
}

impl Profile {
    pub const CURRENT_VERSION: u32 = 1;

    /// A fresh profile: mainnet gateway, dark theme, a single on-device factor source.
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            gateways: Gateways::default(),
            app_preferences: AppPreferences::default(),
            authorized_dapps: Vec::new(),
            factor_sources: vec![FactorSourceMeta::device()],
            security_structures: Vec::new(),
        }
    }

    /// Records (or updates) a dApp authorization, keyed by its definition address.
    pub fn upsert_authorized_dapp(&mut self, dapp: AuthorizedDapp) {
        match self
            .authorized_dapps
            .iter_mut()
            .find(|d| d.dapp_definition_address == dapp.dapp_definition_address)
        {
            Some(existing) => *existing = dapp,
            None => self.authorized_dapps.push(dapp),
        }
    }

    /// Removes a dApp authorization. Returns true if one was removed.
    pub fn forget_authorized_dapp(&mut self, dapp_definition_address: &str) -> bool {
        let before = self.authorized_dapps.len();
        self.authorized_dapps
            .retain(|d| d.dapp_definition_address != dapp_definition_address);
        self.authorized_dapps.len() != before
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::new()
    }
}

/// The active network/gateway plus the user's saved gateways.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Gateways {
    pub current: GatewayConfig,
    pub saved: Vec<GatewayConfig>,
}

impl Default for Gateways {
    fn default() -> Self {
        let mainnet = GatewayConfig::mainnet();
        let stokenet = GatewayConfig::stokenet();
        Self {
            current: mainnet.clone(),
            saved: vec![mainnet, stokenet],
        }
    }
}

impl Gateways {
    /// Switches the active gateway, remembering it in `saved` if new.
    pub fn switch_to(&mut self, gateway: GatewayConfig) {
        if !self.saved.iter().any(|g| g.url == gateway.url) {
            self.saved.push(gateway.clone());
        }
        self.current = gateway;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayConfig {
    pub network: Network,
    pub url: String,
}

impl GatewayConfig {
    pub fn mainnet() -> Self {
        Self {
            network: Network::Mainnet,
            url: "https://mainnet.radixdlt.com".to_string(),
        }
    }

    pub fn stokenet() -> Self {
        Self {
            network: Network::Stokenet,
            url: "https://babylon-stokenet-gateway.radixdlt.com".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppPreferences {
    pub theme: Theme,
    pub security: SecurityPreferences,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            security: SecurityPreferences::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityPreferences {
    pub is_app_lock_enabled: bool,
    pub is_developer_mode_enabled: bool,
}

/// A dApp the user has authorized, and the personas/accounts/data shared with it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizedDapp {
    pub dapp_definition_address: String,
    pub display_name: Option<String>,
    pub origin: String,
    pub authorized_personas: Vec<AuthorizedPersona>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorizedPersona {
    pub identity_address: String,
    pub shared_account_addresses: Vec<String>,
    pub shared_persona_data: PersonaData,
}

/// Metadata describing a factor source (the secret lives in the OS secrets store / on a device).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactorSourceMeta {
    pub id: String,
    pub kind: FactorSourceKind,
    pub label: String,
}

impl FactorSourceMeta {
    pub fn device() -> Self {
        Self {
            id: "device".to_string(),
            kind: FactorSourceKind::Device,
            label: "This device".to_string(),
        }
    }
}

/// The kinds of factor source the wallet can use to sign / authenticate (mirrors the official
/// wallet's factor-source taxonomy; only `Device` is implemented today).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FactorSourceKind {
    /// The seed phrase held on this device, in the OS secrets store.
    Device,
    /// A Ledger hardware wallet.
    LedgerHardware,
    /// A mnemonic the user keeps off-device (entered when needed).
    OffDeviceMnemonic,
    /// A separate password factor.
    Password,
    /// An Arculus card.
    ArculusCard,
}

/// A Security Shield: a matrix of factors arranged into the roles the wallet's Access Controller
/// enforces on-ledger. Primary signs day-to-day transactions; recovery can start a recovery;
/// confirmation confirms it. Factor sources are referenced by their [`FactorSourceMeta::id`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityStructure {
    pub id: String,
    pub label: String,
    pub primary_role: RoleOfFactors,
    pub recovery_role: RoleOfFactors,
    pub confirmation_role: RoleOfFactors,
}

impl SecurityStructure {
    /// A single-factor shield (today's default: one device factor for every role).
    pub fn single_factor(id: impl Into<String>, label: impl Into<String>, factor_id: impl Into<String>) -> Self {
        let factor_id = factor_id.into();
        let role = RoleOfFactors::single(&factor_id);
        Self {
            id: id.into(),
            label: label.into(),
            primary_role: role.clone(),
            recovery_role: role.clone(),
            confirmation_role: role,
        }
    }
}

/// One role within a Security Shield: a threshold over some factors, plus override factors any
/// one of which satisfies the role on its own.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoleOfFactors {
    /// How many of `threshold_factors` are required.
    pub threshold: u8,
    pub threshold_factors: Vec<String>,
    pub override_factors: Vec<String>,
}

impl RoleOfFactors {
    /// A role satisfied by a single factor.
    pub fn single(factor_id: &str) -> Self {
        Self {
            threshold: 1,
            threshold_factors: vec![factor_id.to_string()],
            override_factors: Vec::new(),
        }
    }

    /// Whether a role is well-formed: the threshold is achievable, or an override exists.
    pub fn is_satisfiable(&self) -> bool {
        (self.threshold >= 1 && (self.threshold as usize) <= self.threshold_factors.len())
            || !self.override_factors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_profile_has_sensible_defaults() {
        let profile = Profile::new();
        assert_eq!(profile.version, Profile::CURRENT_VERSION);
        assert_eq!(profile.gateways.current.network, Network::Mainnet);
        assert_eq!(profile.app_preferences.theme.as_str(), "Dark");
        assert!(!profile.app_preferences.security.is_app_lock_enabled);
        assert_eq!(profile.factor_sources.len(), 1);
        assert_eq!(profile.factor_sources[0].kind, FactorSourceKind::Device);
        assert!(profile.authorized_dapps.is_empty());
    }

    #[test]
    fn serde_roundtrip() {
        let profile = Profile::new();
        let json = serde_json::to_string(&profile).unwrap();
        let restored: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, restored);
    }

    #[test]
    fn switching_gateway_adds_and_activates() {
        let mut gateways = Gateways::default();
        let custom = GatewayConfig {
            network: Network::Stokenet,
            url: "https://my-custom-gateway.example".to_string(),
        };
        gateways.switch_to(custom.clone());
        assert_eq!(gateways.current, custom);
        assert!(gateways.saved.iter().any(|g| g.url == custom.url));
        // Switching to an already-saved gateway doesn't duplicate it.
        let saved_count = gateways.saved.len();
        gateways.switch_to(GatewayConfig::mainnet());
        assert_eq!(gateways.saved.len(), saved_count);
        assert_eq!(gateways.current.network, Network::Mainnet);
    }

    #[test]
    fn upsert_and_forget_authorized_dapp() {
        let mut profile = Profile::new();
        let dapp = AuthorizedDapp {
            dapp_definition_address: "account_rdx_dapp".to_string(),
            display_name: Some("Test dApp".to_string()),
            origin: "https://dapp.example".to_string(),
            authorized_personas: vec![],
        };
        profile.upsert_authorized_dapp(dapp.clone());
        assert_eq!(profile.authorized_dapps.len(), 1);

        // Upserting the same dApp updates rather than duplicates.
        let mut updated = dapp.clone();
        updated.display_name = Some("Renamed".to_string());
        profile.upsert_authorized_dapp(updated);
        assert_eq!(profile.authorized_dapps.len(), 1);
        assert_eq!(
            profile.authorized_dapps[0].display_name.as_deref(),
            Some("Renamed")
        );

        assert!(profile.forget_authorized_dapp("account_rdx_dapp"));
        assert!(profile.authorized_dapps.is_empty());
        assert!(!profile.forget_authorized_dapp("account_rdx_dapp"));
    }

    #[test]
    fn single_factor_security_structure_is_satisfiable_and_serdes() {
        let shield = SecurityStructure::single_factor("shield-1", "Default", "device");
        assert!(shield.primary_role.is_satisfiable());
        assert!(shield.recovery_role.is_satisfiable());
        assert!(shield.confirmation_role.is_satisfiable());
        assert_eq!(shield.primary_role.threshold, 1);

        let mut profile = Profile::new();
        profile.security_structures.push(shield);
        let json = serde_json::to_string(&profile).unwrap();
        let restored: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, restored);
    }

    #[test]
    fn role_satisfiability_rules() {
        // threshold larger than available factors, no override -> not satisfiable
        let bad = RoleOfFactors {
            threshold: 2,
            threshold_factors: vec!["a".to_string()],
            override_factors: vec![],
        };
        assert!(!bad.is_satisfiable());

        // an override alone satisfies it
        let with_override = RoleOfFactors {
            threshold: 2,
            threshold_factors: vec!["a".to_string()],
            override_factors: vec!["ledger".to_string()],
        };
        assert!(with_override.is_satisfiable());

        // 2-of-2 threshold is satisfiable
        let two_of_two = RoleOfFactors {
            threshold: 2,
            threshold_factors: vec!["a".to_string(), "b".to_string()],
            override_factors: vec![],
        };
        assert!(two_of_two.is_satisfiable());
    }
}
