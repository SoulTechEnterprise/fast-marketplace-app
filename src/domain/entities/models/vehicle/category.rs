use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    CarOrPickup,
    Motorcycle,
    SportsVehicle,
    Trailer,
    UtilityTrailer,
    Boat,
    CommercialOrIndustrial,
    Other,
}

impl Category {
    pub fn transform(&self) -> &'static str {
        match self {
            Self::CarOrPickup => "Carro/picape",
            Self::Motorcycle => "Motocicleta",
            Self::SportsVehicle => "Veículos para esportes",
            Self::Trailer => "Trailer",
            Self::UtilityTrailer => "Reboque",
            Self::Boat => "Barco",
            Self::CommercialOrIndustrial => "Comercial/industrial",
            Self::Other => "Outro",
        }
    }
}
