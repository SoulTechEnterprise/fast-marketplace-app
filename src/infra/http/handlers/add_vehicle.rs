use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::infra::http::{
    dtos::add_vehicle::{AddVehicleUseCaseRequest, AddVehicleUseCaseResponse},
    setup::AppState,
};

pub async fn add_vehicle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddVehicleUseCaseRequest>,
) -> Result<Json<AddVehicleUseCaseResponse>, StatusCode> {
    let AddVehicleUseCaseRequest { client_id, vehicle } = payload;

    state
        .vehicle_usecase
        .handle(client_id, vehicle)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(AddVehicleUseCaseResponse {}))
}
