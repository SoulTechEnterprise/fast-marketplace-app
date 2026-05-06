use async_trait::async_trait;

use crate::domain::{entities::property::Property, services::error::DomainError};

#[async_trait]
pub trait PropertyService: Sync + Send {
    async fn get(&self, url: String, token: String) -> Result<Property, DomainError>;
}
