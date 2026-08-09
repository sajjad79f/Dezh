mod console;
mod dto;
mod handlers;
mod state;

pub use state::AppState;

use axum::http::Method;
use axum::routing::{delete, get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/", get(handlers::console))
        .route("/api/modules", get(handlers::list_modules))
        .route("/api/commands", get(handlers::list_commands))
        .route("/api/core-services", get(handlers::list_core_services))
        .route("/api/commands/execute", post(handlers::execute_command))
        .route(
            "/api/firewall/rules",
            get(handlers::list_firewall_rules).post(handlers::create_firewall_rule),
        )
        .route(
            "/api/firewall/rules/{id}",
            delete(handlers::delete_firewall_rule),
        )
        .layer(cors)
        .with_state(state)
}

pub async fn serve(state: AppState, addr: &str) -> std::io::Result<()> {
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web Console listening on http://{addr}");
    axum::serve(listener, app).await
}