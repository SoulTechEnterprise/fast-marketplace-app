use std::sync::Arc;

use app::{
    application::usecases::add_property::AddPropertyUseCase,
    infra::{
        http::{routes::routes, setup::AppState},
        repositories::image::ImageRepositoryImpl,
        services::{
            property::PropertyServiceApi, webscraping::marketplace::FacebookMarketplaceService,
        },
    },
};
use axum::serve;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let image_repository = Arc::new(ImageRepositoryImpl::new());
    let property_service = Arc::new(PropertyServiceApi::new());
    let webscraping_marketplace_service = Arc::new(FacebookMarketplaceService::new());

    let property_usecase = Arc::new(AddPropertyUseCase::new(
        image_repository,
        webscraping_marketplace_service.clone(),
        property_service,
    ));

    let state = Arc::new(AppState {
        auth_marketplace: webscraping_marketplace_service,
        property_usecase,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app_router = routes(state).layer(cors);

    let port = "3001";
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("🚀 server running on port - {}!", port);

    serve(listener, app_router).await.unwrap();
}
