use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::{
    entities::property::Property,
    services::{error::DomainError, property::PropertyService},
};

// ── DTO para desembrulhar o wrapper { "property": { ... } } ──────────────────

#[derive(Serialize, Deserialize)]
struct PropertyRequest {
    url: String,
}

#[derive(Deserialize)]
struct PropertyResponse {
    property: Property,
}

// ── Implementação ─────────────────────────────────────────────────────────────

pub struct PropertyServiceApi {
    client: Client,
}

impl PropertyServiceApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl PropertyService for PropertyServiceApi {
    async fn get(&self, url: String, token: String) -> Result<Property, DomainError> {
        let base_url = if cfg!(debug_assertions) {
            "http://localhost:3000"
        } else {
            "https://fast-marketplace-backend.soultech.agency"
        };

        let endpoint = format!("{}/property", base_url);

        let payload = PropertyRequest { url };

        let response = self
            .client
            .post(&endpoint)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(|_| DomainError::RuleViolation)?;

        let wrapper = response
            .json::<PropertyResponse>()
            .await
            .map_err(|_| DomainError::RuleViolation)?;

        Ok(wrapper.property)
    }
}
