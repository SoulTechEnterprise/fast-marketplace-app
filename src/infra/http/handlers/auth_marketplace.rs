use crate::infra::http::{dtos::auth_marketplace::AuthMarketplaceUseCaseRequest, setup::AppState};
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;

pub async fn auth_marketplace(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthMarketplaceUseCaseRequest>,
) -> StatusCode {
    let AuthMarketplaceUseCaseRequest { client_id } = payload;

    state.auth_marketplace.login(client_id).await.unwrap();
    StatusCode::OK
}
