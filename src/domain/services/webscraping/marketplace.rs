use async_trait::async_trait;

use crate::domain::{
    entities::{property::Property, vehicle::Vehicle},
    services::error::DomainError,
};

#[async_trait]
pub trait WebscrapingMarketplaceService: Send + Sync {
    async fn add_property(&self, entity: Property, client_id: String) -> Result<(), DomainError>;
    async fn add_vehicle(&self, entity: Vehicle, client_id: String) -> Result<(), DomainError>;
}
