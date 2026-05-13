use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::env;

use crate::domain::{
    entities::vehicle::Vehicle,
    services::{error::DomainError, vehicle::VehicleService},
};

// ── DTO para desembrulhar o wrapper { "property": { ... } } ──────────────────

#[derive(Serialize, Deserialize)]
struct VehicleRequest {
    url: String,
}

#[derive(Deserialize)]
struct VehicleResponse {
    vehicle: Vehicle,
}

// ── Implementação ─────────────────────────────────────────────────────────────

pub struct VehicleServiceApi {
    client: Client,
    base_url: String,
}

impl VehicleServiceApi {
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
impl VehicleService for VehicleServiceApi {
    async fn get(&self, url: String, token: String) -> Result<Vehicle, DomainError> {
        let endpoint = format!("{}/vehicle", self.base_url);

        let payload = VehicleRequest { url };

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
                    .json::<VehicleResponse>()
                    .await
                    .map_err(|_| DomainError::RuleViolation)?;

                Ok(wrapper.vehicle)
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(DomainError::LimitReached),
            reqwest::StatusCode::BAD_REQUEST => Err(DomainError::RuleViolation),
            _ => Err(DomainError::RuleViolation),
        }
    }
}
