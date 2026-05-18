use serde::{Deserialize, Serialize};

use crate::domain::entities::property::Property;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPropertyUseCaseRequest {
    pub property: Property,
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPropertyUseCaseResponse {}
