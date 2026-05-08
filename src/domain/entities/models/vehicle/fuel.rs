use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]

pub enum Fuel {
    Diesel,
    Electric,
    Gasoline,
    Flex,
    Hybrid,
    PlugInHybrid,
    Other,
}

impl Fuel {
    pub fn transform(&self) -> &'static str {
        match self {
            Self::Diesel => "Diesel",
            Self::Electric => "Elétrico",
            Self::Gasoline => "Gasolina",
            Self::Flex => "Flex",
            Self::Hybrid => "Híbrido",
            Self::PlugInHybrid => "Híbrido plug-in",
            Self::Other => "Outro",
        }
    }
}
