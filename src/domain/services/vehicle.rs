use async_trait::async_trait;

use crate::domain::{entities::vehicle::Vehicle, services::error::DomainError};

#[async_trait]
pub trait VehicleService: Sync + Send {
    async fn get(&self, url: String, token: String) -> Result<Vehicle, DomainError>;
}
