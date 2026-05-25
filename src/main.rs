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
    let addr: SocketAddr = match format!("127.0.0.1:{}", port).parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Falha ao analisar endereço de rede: {}", e);
            std::process::exit(1);
        }
    };

    let socket = match TcpSocket::new_v4() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Falha ao criar socket TCP: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = socket.set_reuseaddr(true) {
        eprintln!("Falha ao configurar reuseaddr: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = socket.bind(addr) {
        eprintln!(
            "Falha ao associar a porta {}: {}. Certifique-se de que outra instância do aplicativo não esteja rodando.",
            port, e
        );
        std::process::exit(1);
    }

    let listener = match socket.listen(1024) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Falha ao iniciar escuta na porta {}: {}", port, e);
            std::process::exit(1);
        }
    };

    println!("🚀 server running on port - {}!", port);

    if let Err(e) = serve(listener, app_router)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("Erro na execução do servidor HTTP: {}", e);
    }
}
