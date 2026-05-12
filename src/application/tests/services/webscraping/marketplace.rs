use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::domain::{
    entities::{property::Property, vehicle::Vehicle},
    services::{error::DomainError, webscraping::marketplace::WebscrapingMarketplaceService},
};

pub struct InMemoryWebscrapingMarketplaceService {
    pub properties: Mutex<Vec<Property>>,
    pub vehicles: Mutex<Vec<Vehicle>>,
    pub sessions: Mutex<HashSet<String>>,
}

impl InMemoryWebscrapingMarketplaceService {
    pub fn new() -> Self {
        Self {
            properties: Mutex::new(Vec::new()),
            vehicles: Mutex::new(Vec::new()),
            sessions: Mutex::new(HashSet::new()),
        }
    }
}

impl Default for InMemoryWebscrapingMarketplaceService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebscrapingMarketplaceService for InMemoryWebscrapingMarketplaceService {
    async fn add_property(&self, entity: Property, _client_id: String) -> Result<(), DomainError> {
        self.properties.lock().unwrap().push(entity);
        Ok(())
    }

    async fn add_vehicle(&self, entity: Vehicle, _client_id: String) -> Result<(), DomainError> {
        self.vehicles.lock().unwrap().push(entity);
        Ok(())
    }

    async fn signin(&self, client_id: String) -> Result<(), DomainError> {
        self.sessions.lock().unwrap().insert(client_id);
        Ok(())
    }

    async fn signout(&self, client_id: String) -> Result<(), DomainError> {
        self.sessions.lock().unwrap().remove(&client_id);
        Ok(())
    }

    async fn get_account(&self, client_id: String) -> Result<bool, DomainError> {
        let is_signed_in = self.sessions.lock().unwrap().contains(&client_id);
        Ok(is_signed_in)
    }
}
