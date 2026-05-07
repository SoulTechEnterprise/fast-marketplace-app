use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddVehicleUseCaseRequest {
    pub url: String,
    pub token: String,
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddVehicleUseCaseResponse {}
