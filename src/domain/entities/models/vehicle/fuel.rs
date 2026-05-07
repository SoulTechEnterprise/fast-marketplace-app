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
