use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{
    application::error::UseCasesError,
    domain::services::error::DomainError,
    infra::http::{
        dtos::add_vehicle::{AddVehicleUseCaseRequest, AddVehicleUseCaseResponse},
        setup::AppState,
    },
};

pub async fn add_vehicle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddVehicleUseCaseRequest>,
) -> Result<Json<AddVehicleUseCaseResponse>, StatusCode> {
    let AddVehicleUseCaseRequest {
        url,
        token,
        client_id,
    } = payload;

    state
        .vehicle_usecase
        .handle(url, token, client_id)
        .await
        .map_err(|err| match err {
            UseCasesError::Domain(DomainError::LimitReached) => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::BAD_REQUEST,
        })?;

    Ok(Json(AddVehicleUseCaseResponse {}))
}
