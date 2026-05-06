use async_trait::async_trait;

use crate::domain::{
    entities::{
        models::property::{category::Category, model::Model},
        property::Property,
    },
    services::{error::DomainError, property::PropertyService},
};

pub struct InMemoryPropertyService {
    pub property: Property,
}

impl InMemoryPropertyService {
    pub fn new() -> Self {
        Self {
            property: Property::new(
                vec![
                    "https://example.com/image1.jpg".to_string(),
                    "https://example.com/image2.jpg".to_string(),
                ],
                Model::Sale,
                Category::Apartment,
                3,
                2,
                50000,
                "Rua Example, 123 - São Paulo, SP".to_string(),
                "Apartamento lindo com vista para o mar".to_string(),
                80,
                500,
                800,
                1,
            ),
        }
    }
}

#[async_trait]
impl PropertyService for InMemoryPropertyService {
    async fn get(&self, _url: String, _token: String) -> Result<Property, DomainError> {
        return Ok(self.property.clone());
    }
}
