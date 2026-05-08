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

impl BodyStyle {
    pub fn transform(&self) -> &'static str {
        match self {
            Self::Coupe => "Cupê",
            Self::Pickup => "Picape",
            Self::Sedan => "Sedã",
            Self::Hatchback => "Hatch",
            Self::Suv => "SUV",
            Self::Convertible => "Conversível",
            Self::StationWagon => "Station wagon",
            Self::Minivan => "Minivan",
            Self::CompactCar => "Carro compacto",
            Self::Other => "Outro",
        }
    }
}
