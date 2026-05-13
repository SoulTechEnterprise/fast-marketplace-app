use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::env;

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
    base_url: String,
}

impl PropertyServiceApi {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        let base_url = if args.len() > 1 {
            args[1].clone()
        } else {
            "https://fast-marketplace-backend.soultech.agency".to_string()
        };

        Self {
            client: Client::new(),
            base_url,
        }
    }
}

#[async_trait]
impl PropertyService for PropertyServiceApi {
    async fn get(&self, url: String, token: String) -> Result<Property, DomainError> {
        let endpoint = format!("{}/property", self.base_url);

        let payload = PropertyRequest { url };

        let response = self
            .client
            .post(&endpoint)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(|_| DomainError::RuleViolation)?;

        match response.status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {
                let wrapper = response
                    .json::<PropertyResponse>()
                    .await
                    .map_err(|_| DomainError::RuleViolation)?;

                Ok(wrapper.property)
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(DomainError::LimitReached),
            reqwest::StatusCode::BAD_REQUEST => Err(DomainError::RuleViolation),
            _ => Err(DomainError::RuleViolation),
        }
    }
}
