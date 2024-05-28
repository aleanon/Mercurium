use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Details {
    package_address: String,
    blueprint_name: String,
    blueprint_version: String,
    state: State,
    role_assignments: RoleAssignements,
    #[serde(rename = "type")]
    type_field: String,
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
    pub assignment: Assignment,
    pub updater_roles: Vec<UpdaterRole>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleKey {
    pub module: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Assignment {
    pub resolution: String,
    pub explicit_rule: Option<Rule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdaterRole {
    pub module: String,
    pub name: String,
}
