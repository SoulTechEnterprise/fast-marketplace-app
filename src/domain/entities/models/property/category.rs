use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Apartment,
    House,
}

impl Category {
    pub fn transform(&self) -> &'static str {
        match self {
            Self::Apartment => "Apartamento",
            Self::House => "Casa",
        }
    }
}
