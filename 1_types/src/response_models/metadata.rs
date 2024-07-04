use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaDataStringArrayValue {
    pub values: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaDataStringValue {
    pub value: String,
}
