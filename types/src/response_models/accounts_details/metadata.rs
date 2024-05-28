use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
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
    pub element_kind: Option<String>,
    pub elements: Option<Vec<Element>>,
    pub value: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    pub variant_id: u32,
    pub fields: Vec<ElementField>,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementField {
    pub element_kind: String,
    pub hex: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Typed {
    pub values: Option<Vec<TypedValue>>,
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypedValue {
    pub hash_hex: String,
    pub key_hash_type: String,
}
