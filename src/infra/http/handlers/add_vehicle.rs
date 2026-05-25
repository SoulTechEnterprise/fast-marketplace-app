use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::infra::http::{
    dtos::add_vehicle::{AddVehicleUseCaseRequest, AddVehicleUseCaseResponse},
    setup::AppState,
};

pub async fn add_vehicle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddVehicleUseCaseRequest>,
) -> Result<Json<AddVehicleUseCaseResponse>, (StatusCode, Json<serde_json::Value>)> {
    let AddVehicleUseCaseRequest { client_id, vehicle } = payload;

    state
        .vehicle_usecase
        .handle(client_id, vehicle)
        .await
        .map_err(|e| {
            let msg = match e {
                crate::application::error::UseCasesError::Domain(crate::domain::services::error::DomainError::AutomationError(m)) => m,
                other => other.to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg })))
        })?;

    Ok(Json(AddVehicleUseCaseResponse {}))
}
