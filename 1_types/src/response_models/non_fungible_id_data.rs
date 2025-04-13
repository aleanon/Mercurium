use deps::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NFIdData {
    pub fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    pub value: String,
    pub kind: String,
    pub type_name: Option<String>,
    pub field_name: String,
}
