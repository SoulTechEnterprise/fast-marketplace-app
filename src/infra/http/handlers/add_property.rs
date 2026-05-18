use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::infra::http::{
    dtos::add_property::{AddPropertyUseCaseRequest, AddPropertyUseCaseResponse},
    setup::AppState,
};

pub async fn add_property(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddPropertyUseCaseRequest>,
) -> Result<Json<AddPropertyUseCaseResponse>, StatusCode> {
    let AddPropertyUseCaseRequest {
        client_id,
        property,
    } = payload;

    state
        .property_usecase
        .handle(client_id, property)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(AddPropertyUseCaseResponse {}))
}
