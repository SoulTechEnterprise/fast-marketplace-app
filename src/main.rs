use std::sync::Arc;

use app::{
    application::usecases::{
        add_property::AddPropertyUseCase, add_vehicle::AddVehicleUseCase,
        get_marketplace::GetMarketplaceUseCase, signin_marketplace::SignInMarketplaceUseCase,
        signout_marketplace::SignOutMarketplaceUseCase,
    },
    infra::{
        http::{routes::routes, setup::AppState},
        repositories::image::ImageRepositoryImpl,
        services::{
            property::PropertyServiceApi, vehicle::VehicleServiceApi,
            webscraping::marketplace::FacebookMarketplaceService,
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
    let vehicle_service = Arc::new(VehicleServiceApi::new());
    let webscraping_marketplace_service = Arc::new(FacebookMarketplaceService::new());

    let property_usecase = Arc::new(AddPropertyUseCase::new(
        image_repository.clone(),
        webscraping_marketplace_service.clone(),
        property_service,
    ));

    let vehicle_usecase = Arc::new(AddVehicleUseCase::new(
        image_repository,
        webscraping_marketplace_service.clone(),
        vehicle_service,
    ));

    let signin_marketplace_usecase = Arc::new(SignInMarketplaceUseCase::new(
        webscraping_marketplace_service.clone(),
    ));

    let signout_marketplace_usecase = Arc::new(SignOutMarketplaceUseCase::new(
        webscraping_marketplace_service.clone(),
    ));

    let get_marketplace_usecase =
        Arc::new(GetMarketplaceUseCase::new(webscraping_marketplace_service));

    let state = Arc::new(AppState {
        property_usecase,
        vehicle_usecase,
        signin_marketplace_usecase,
        signout_marketplace_usecase,
        get_marketplace_usecase,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app_router = routes(state).layer(cors);

    let port = "15137";
    let listener = TcpListener::bind(format!("[::]:{}", port)).await.unwrap();

    println!("🚀 server running on port - {}!", port);

    serve(listener, app_router).await.unwrap();
}
