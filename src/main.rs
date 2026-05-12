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

use self_update::cargo_crate_version;

fn check_for_updates() -> Result<(), Box<dyn std::error::Error>> {
    let executable_name = if cfg!(target_os = "windows") {
        "app-windows.exe"
    } else if cfg!(target_os = "macos") {
        "app-macos"
    } else {
        "app-linux"
    };

    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("SoulTechEnterprise")
        .repo_name("fast-marketplace-app")
        .build()?
        .fetch()?;

    if releases.is_empty() {
        return Ok(());
    }

    let latest_release = &releases[0];

    let is_greater =
        self_update::version::bump_is_greater(cargo_crate_version!(), &latest_release.version)?;

    if !is_greater {
        return Ok(());
    }

    let asset = latest_release
        .asset_for(executable_name, None)
        .ok_or("Asset nao encontrado")?;

    let tmp_dir = tempfile::Builder::new().prefix("update").tempdir()?;
    let tmp_file_path = tmp_dir.path().join(executable_name);
    let mut tmp_file = std::fs::File::create(&tmp_file_path)?;

    self_update::Download::from_url(&asset.download_url)
        .show_progress(true)
        .download_to(&mut tmp_file)?;

    self_replace::self_replace(&tmp_file_path)?;

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

    let port = "3001";
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    println!("🚀 server running on port - {}!", port);

    serve(listener, app_router).await.unwrap();
}
