use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NFIdData {
    pub fields: Vec<crate::non_fungibles::NFData>,
}
