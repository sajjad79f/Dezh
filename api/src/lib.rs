mod console;
mod dto;
mod handlers;
mod state;

pub use state::AppState;

use axum::routing::{get, post};
use axum::Router;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::console))
        .route("/api/modules", get(handlers::list_modules))
        .route("/api/commands", get(handlers::list_commands))
        .route("/api/core-services", get(handlers::list_core_services))
        .route("/api/commands/execute", post(handlers::execute_command))
        .with_state(state)
}

/// Starts the HTTP server and blocks until it stops.
/// Must be called from within a Tokio runtime.
pub async fn serve(state: AppState, addr: &str) -> std::io::Result<()> {
    let app = router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Web Console listening on http://{addr}");

    axum::serve(listener, app).await
}
