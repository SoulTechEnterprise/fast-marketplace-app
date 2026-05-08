use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Condition {
    Excellent,
    VeryGood,
    Good,
    Fair,
    Poor,
}

impl Condition {
    pub fn transform(&self) -> &'static str {
        match self {
            Self::Excellent => "Excelente",
            Self::VeryGood => "Muito bom",
            Self::Good => "Bom",
            Self::Fair => "Razoável",
            Self::Poor => "Ruim",
        }
    }
}
