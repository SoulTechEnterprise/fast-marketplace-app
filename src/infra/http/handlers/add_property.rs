use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::infra::http::{
    dtos::add_property::{AddPropertyUseCaseRequest, AddPropertyUseCaseResponse},
    setup::AppState,
};

pub async fn add_property(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddPropertyUseCaseRequest>,
) -> Result<Json<AddPropertyUseCaseResponse>, (StatusCode, Json<serde_json::Value>)> {
    let AddPropertyUseCaseRequest {
        client_id,
        property,
    } = payload;

    state
        .property_usecase
        .handle(client_id, property)
        .await
        .map_err(|e| {
            let msg = match e {
                crate::application::error::UseCasesError::Domain(crate::domain::services::error::DomainError::AutomationError(m)) => m,
                other => other.to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg })))
        })?;

    Ok(Json(AddPropertyUseCaseResponse {}))
}
