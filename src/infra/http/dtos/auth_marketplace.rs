use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthMarketplaceUseCaseRequest {
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthMarketplaceUseCaseResponse {}
