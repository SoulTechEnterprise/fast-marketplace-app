use std::net::SocketAddr;
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
        services::webscraping::marketplace::FacebookMarketplaceService,
    },
};
use axum::{http::HeaderValue, serve};
use tokio::net::TcpSocket;
use tower_http::cors::{Any, CorsLayer};

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap();
}

#[tokio::main]
async fn main() {
    #[cfg(not(debug_assertions))]
    if let Err(e) = check_and_update() {
        eprintln!("Falha ao verificar atualização: {}", e);
    }

    let image_repository = Arc::new(ImageRepositoryImpl::new());
    let webscraping_marketplace_service = Arc::new(FacebookMarketplaceService::new());

    let property_usecase = Arc::new(AddPropertyUseCase::new(
        image_repository.clone(),
        webscraping_marketplace_service.clone(),
    ));

    let vehicle_usecase = Arc::new(AddVehicleUseCase::new(
        image_repository,
        webscraping_marketplace_service.clone(),
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

    let allowed_origins = [
        "http://localhost:4000".parse::<HeaderValue>().unwrap(),
        "https://soultech.agency".parse::<HeaderValue>().unwrap(),
        "https://fast-marketplace-dev-frontend.soultech.agency"
            .parse::<HeaderValue>()
            .unwrap(),
        "https://fast-marketplace-frontend.soultech.agency"
            .parse::<HeaderValue>()
            .unwrap(),
    ];

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods(Any)
        .allow_headers(Any);

    let app_router = routes(state).layer(cors);

    let port = "15137";
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();

    let socket = TcpSocket::new_v4().unwrap();
    socket.set_reuseaddr(true).unwrap();
    socket.bind(addr).unwrap();

    let listener = socket.listen(1024).unwrap();

    println!("🚀 server running on port - {}!", port);

    serve(listener, app_router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

pub fn check_and_update() -> Result<(), Box<dyn std::error::Error>> {
    let bin_name = if cfg!(target_os = "windows") {
        "automatize-marketplace-windows.exe"
    } else if cfg!(target_os = "macos") {
        "automatize-marketplace-macos"
    } else {
        "automatize-marketplace-linux"
    };

    let status = self_update::backends::github::Update::configure()
        .repo_owner("SoulTechEnterprise")
        .repo_name("fast-marketplace-app")
        .bin_name(bin_name)
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_download_progress(true)
        .build()?
        .update()?;

    if status.updated() {
        println!(
            "Atualizado para a versão {}! Reiniciando...",
            status.version()
        );
        std::process::exit(0);
    } else {
        println!("Já está na versão mais recente.");
    }

    Ok(())
}
