use std::sync::Mutex;

use async_trait::async_trait;

use crate::domain::{
    entities::{item::Item, property::Property, vehicle::Vehicle},
    services::{error::DomainError, webscraping::marketplace::WebscrapingMarketplaceService},
};

pub struct InMemoryWebscrapingMarketplaceService {
    pub properties: Mutex<Vec<Property>>,
    pub vehicles: Mutex<Vec<Vehicle>>,
    pub items: Mutex<Vec<Item>>,
}

impl InMemoryWebscrapingMarketplaceService {
    pub fn new() -> Self {
        Self {
            properties: Mutex::new(Vec::new()),
            vehicles: Mutex::new(Vec::new()),
            items: Mutex::new(Vec::new()),
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

    async fn add_item(&self, entity: Item, _client_id: String) -> Result<(), DomainError> {
        self.items.lock().unwrap().push(entity);
        Ok(())
    }
}
