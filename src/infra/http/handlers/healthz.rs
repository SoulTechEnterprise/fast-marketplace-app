use axum::{Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthzResponse {
    status: String,
    version: &'static str,
}

pub async fn healthz() -> (StatusCode, Json<HealthzResponse>) {
    (
        StatusCode::OK,
        Json(HealthzResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION"),
        }),
    )
}
