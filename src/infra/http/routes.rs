use std::sync::Arc;

use axum::{Router, routing::post};

use crate::infra::http::{
    handlers::{add_property::add_property, auth_marketplace::auth_marketplace},
    setup::AppState,
};

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/property", post(add_property))
        .route("/auth", post(auth_marketplace))
        .with_state(state)
}
