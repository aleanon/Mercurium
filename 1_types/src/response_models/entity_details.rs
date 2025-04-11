use deps_two::*;

use serde::{Deserialize, Serialize};

use super::ledger_state::LedgerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityDetailsResponse {
    pub ledger_state: LedgerState,
    pub items: Vec<Entity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub address: String,
    pub fungible_resources: FungibleResources,
    pub non_fungible_resources: NonFungibleResources,
    pub metadata: Metadata,
    //explicit_metadata: ExplicitMetadata,
    pub details: Details,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleResources {
    pub total_count: u32,
    pub items: Vec<FungibleResource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FungibleResource {
    pub vaults: Vaults,
    pub aggregation_level: String,
    pub resource_address: String,
    pub explicit_metadata: ExplicitMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vaults {
    pub total_count: u32,
    pub items: Vec<Vault>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vault {
    pub vault_address: String,
    pub amount: String,
    pub last_updated_at_state_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExplicitMetadata {
    pub total_count: u32,
    pub items: Vec<MetadataItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataItem {
    pub key: String,
    pub value: Value,
    pub is_locked: bool,
    pub last_updated_at_state_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Value {
    pub raw_hex: String,
    pub programmatic_json: ProgrammaticJson,
    pub typed: Typed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgrammaticJson {
    pub kind: String,
    pub variant_id: u32,
    pub fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    pub kind: String,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Typed {
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleResources {
    pub total_count: u32,
    pub items: Vec<NonFungibleResource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungibleResource {
    pub vaults: NFTVaults,
    pub aggregation_level: String,
    pub resource_address: String,
    pub explicit_metadata: ExplicitMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTVaults {
    pub total_count: u32,
    pub items: Vec<NFTVault>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTVault {
    pub total_count: u32,
    pub items: Vec<String>,
    pub vault_address: String,
    pub last_updated_at_state_version: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub total_count: u32,
    pub items: Vec<MetadataItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Details {
    pub total_supply: Option<String>,
    //pub divisibility: Option<u32>,
    //package_address: String,
    //blueprint_name: String,
    //blueprint_version: String,
    //state: Option<State>,
    //role_assignments: RoleAssignements,
    //#[serde(rename = "type")]
    //type_field: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub default_deposit_rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleAssignements {
    pub owner: Owner,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub rule: Rule,
    pub updater: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "type")]
    pub type_field: String,
    pub access_rule: AccessRule,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessRule {
    #[serde(rename = "type")]
    pub type_field: String,
    pub proof_rule: ProofRule,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofRule {
    #[serde(rename = "type")]
    pub type_field: String,
    pub requirement: Requirement,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Requirement {
    #[serde(rename = "type")]
    pub type_field: String,
    pub non_fungible: NonFungible,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonFungible {
    pub local_id: LokalId,
    pub resource_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LokalId {
    pub id_type: String,
    pub sbor_hex: String,
    pub simple_rep: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub role_key: RoleKey,
    //assignment: Assignement,
    pub updater_roles: Vec<UpdaterRole>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleKey {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Assignement {
    pub resolution: String,
    pub explicit_rule: Rule,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdaterRole {
    pub module: String,
    pub name: String,
}
