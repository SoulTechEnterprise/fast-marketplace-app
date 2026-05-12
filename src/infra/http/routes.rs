use std::sync::Arc;

use axum::{Router, routing::get, routing::post};

use crate::infra::http::{
    handlers::{
        add_property::add_property, add_vehicle::add_vehicle, get_marketplace::get_marketplace,
        healthz::healthz, signin_marketplace::signin_marketplace,
        signout_marketplace::signout_marketplace,
    },
    setup::AppState,
};

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/property", post(add_property))
        .route("/vehicle", post(add_vehicle))
        .route("/sign-in", post(signin_marketplace))
        .route("/sign-out", post(signout_marketplace))
        .route("/marketplace", post(get_marketplace))
        .route("/healthz", get(healthz))
        .with_state(state)
}
