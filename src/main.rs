use std::sync::Arc;

use app::{
    application::usecases::{add_property::AddPropertyUseCase, add_vehicle::AddVehicleUseCase},
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

use self_update::cargo_crate_version;

fn check_for_updates() -> Result<(), Box<dyn std::error::Error>> {
    let executable_name = if cfg!(target_os = "windows") {
        "app-windows.exe"
    } else if cfg!(target_os = "macos") {
        "app-macos"
    } else {
        "app-linux"
    };

    self_update::backends::github::Update::configure()
        .repo_owner("SoulTechEnterprise")
        .repo_name("fast-marketplace-app")
        .bin_name(executable_name)
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    Ok(())
}

#[tokio::main]
async fn main() {
    #[cfg(not(debug_assertions))]
    if let Err(e) = check_for_updates() {
        eprintln!("error on update: {}", e);
    }

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

    let state = Arc::new(AppState {
        auth_marketplace: webscraping_marketplace_service,
        property_usecase,
        vehicle_usecase,
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
