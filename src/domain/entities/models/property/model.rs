use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Sale,
    Rent,
}

impl Model {
    pub fn transform(&self) -> &'static str {
        match self {
            Self::Sale => "À venda",
            Self::Rent => "Aluguel",
        }
    }
}
