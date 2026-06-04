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
        logger,
        repositories::image::ImageRepositoryImpl,
        services::webscraping::marketplace::FacebookMarketplaceService,
    },
};
use axum::{http::{HeaderValue, Method}, serve};
use tokio::net::TcpSocket;
use tower_http::cors::CorsLayer;

// ─── Constantes de configuração do servidor ─────────────────────────────────

const SERVER_PORT: &str = "15137";
const SERVER_HOST: &str = "127.0.0.1";

// ─── Origens permitidas para CORS ───────────────────────────────────────────

const ALLOWED_ORIGINS: [&str; 4] = [
    "http://localhost:4000",
    "https://soultech.agency",
    "https://fast-marketplace-dev-frontend.soultech.agency",
    "https://fast-marketplace-frontend.soultech.agency",
];

// ─── Sinal de encerramento gracioso ─────────────────────────────────────────

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    logger::print_shutdown();
}

// ─── Ponto de entrada ───────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // ── Exibir banner profissional ──────────────────────────────────────
    logger::print_banner(env!("CARGO_PKG_VERSION"), SERVER_PORT);
    logger::separator();

    // ── Inicializar dependências ────────────────────────────────────────
    logger::info("Inicializando serviços...");

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

    logger::info("Serviços inicializados com sucesso");

    // ── Configurar CORS ─────────────────────────────────────────────────
    let allowed_origins: Vec<HeaderValue> = ALLOWED_ORIGINS
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);

    let app_router = routes(state).layer(cors);

    // ── Iniciar servidor TCP ────────────────────────────────────────────
    let addr: SocketAddr = match format!("{}:{}", SERVER_HOST, SERVER_PORT).parse() {
        Ok(addr) => addr,
        Err(e) => {
            logger::error(&format!("Falha ao analisar endereço de rede: {}", e));
            std::process::exit(1);
        }
    };

    let socket = match TcpSocket::new_v4() {
        Ok(s) => s,
        Err(e) => {
            logger::error(&format!("Falha ao criar socket TCP: {}", e));
            std::process::exit(1);
        }
    };

    if let Err(e) = socket.set_reuseaddr(true) {
        logger::error(&format!("Falha ao configurar reuseaddr: {}", e));
        std::process::exit(1);
    }

    if let Err(e) = socket.bind(addr) {
        logger::error(&format!(
            "Falha ao associar a porta {}: {}. Certifique-se de que outra instância do aplicativo não esteja rodando.",
            SERVER_PORT, e
        ));
        std::process::exit(1);
    }

    let listener = match socket.listen(1024) {
        Ok(l) => l,
        Err(e) => {
            logger::error(&format!("Falha ao iniciar escuta na porta {}: {}", SERVER_PORT, e));
            std::process::exit(1);
        }
    };

    logger::info(&format!(
        "Servidor HTTP rodando em http://{}:{}",
        SERVER_HOST, SERVER_PORT
    ));
    logger::separator();

    // ── Executar servidor ───────────────────────────────────────────────
    if let Err(e) = serve(listener, app_router)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        logger::error(&format!("Erro na execução do servidor HTTP: {}", e));
    }
}
