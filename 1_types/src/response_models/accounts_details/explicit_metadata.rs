use serde::{Deserialize, Serialize};

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
    pub variant_id: u32,
    pub fields: Vec<Field>,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    pub value: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Typed {
    pub value: String,
    #[serde(rename = "type")]
    pub type_field: String,
}
