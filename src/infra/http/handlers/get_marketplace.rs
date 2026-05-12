use crate::infra::http::{
    dtos::marketplace::{MarketplaceUseCaseRequest, MarketplaceUseCaseResponse},
    setup::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;

pub async fn get_marketplace(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MarketplaceUseCaseRequest>,
) -> Result<Json<MarketplaceUseCaseResponse>, StatusCode> {
    let MarketplaceUseCaseRequest { client_id } = payload;

    let status = state
        .get_marketplace_usecase
        .handle(client_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(MarketplaceUseCaseResponse { status }))
}
