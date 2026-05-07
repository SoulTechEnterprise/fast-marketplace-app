use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BodyStyle {
    Coupe,
    Pickup,
    Sedan,
    Hatchback,
    Suv,
    Convertible,
    StationWagon,
    Minivan,
    CompactCar,
    Other,
}
