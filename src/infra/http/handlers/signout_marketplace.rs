use crate::infra::http::{dtos::marketplace::MarketplaceUseCaseRequest, setup::AppState};
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;

pub async fn signout_marketplace(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MarketplaceUseCaseRequest>,
) -> Result<StatusCode, StatusCode> {
    let MarketplaceUseCaseRequest { client_id } = payload;

    state
        .signout_marketplace_usecase
        .handle(client_id)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::OK)
}
