use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceUseCaseRequest {
    pub client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketplaceUseCaseResponse {
    pub status: bool,
}
