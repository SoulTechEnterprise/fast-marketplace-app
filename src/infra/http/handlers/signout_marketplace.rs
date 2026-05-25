use crate::infra::http::{dtos::marketplace::MarketplaceUseCaseRequest, setup::AppState};
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;

pub async fn signout_marketplace(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MarketplaceUseCaseRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let MarketplaceUseCaseRequest { client_id } = payload;

    state
        .signout_marketplace_usecase
        .handle(client_id)
        .await
        .map_err(|e| {
            let msg = match e {
                crate::application::error::UseCasesError::Domain(crate::domain::services::error::DomainError::AutomationError(m)) => m,
                other => other.to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg })))
        })?;

    Ok(StatusCode::OK)
}
