use async_trait::async_trait;

use crate::domain::{
    entities::{
        models::vehicle::{
            bodystyle::BodyStyle, category::Category, condition::Condition, fuel::Fuel,
            manufacturer::Manufacturer,
        },
        vehicle::Vehicle,
    },
    services::{error::DomainError, vehicle::VehicleService},
};

pub struct InMemoryVehicleService {
    pub vehicle: Vehicle,
}

impl InMemoryVehicleService {
    pub fn new() -> Self {
        Self {
            vehicle: Vehicle::new(
                Category::CarOrPickup,
                vec![
                    "https://cdn.example.com/porsche_frente.jpg".to_string(),
                    "https://cdn.example.com/porsche_painel.jpg".to_string(),
                ],
                "Avenida Brigadeiro Faria Lima, 3000 - São Paulo, SP".to_string(),
                2023,
                Manufacturer::Porsche,
                "911 Carrera S".to_string(),
                5500,
                BodyStyle::Coupe,
                950000,
                Condition::Excellent,
                Fuel::Gasoline,
                "Veículo impecável, único dono. Pacote Sport Chrono, revisões na concessionária e PPF frontal aplicado.".to_string(),
            ),
        }
    }
}

#[async_trait]
impl VehicleService for InMemoryVehicleService {
    async fn get(&self, _url: String, _token: String) -> Result<Vehicle, DomainError> {
        return Ok(self.vehicle.clone());
    }
}
