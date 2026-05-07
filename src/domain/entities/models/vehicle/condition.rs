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
