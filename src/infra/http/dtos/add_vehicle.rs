use serde::{Deserialize, Serialize};

use crate::domain::entities::vehicle::Vehicle;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddVehicleUseCaseRequest {
    pub client_id: String,
    pub vehicle: Vehicle,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddVehicleUseCaseResponse {}
