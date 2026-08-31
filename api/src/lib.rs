mod console;
mod dto;
mod handlers;
mod state;
mod auth;

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
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/users", get(handlers::list_users).post(handlers::create_user))
        .route(
            "/api/identities",
            get(handlers::list_identities).post(handlers::create_identity),
        )
        .route(
            "/api/accounting/sessions",
            get(handlers::list_active_sessions),
        )
        .route(
            "/api/accounting/sessions/start",
            post(handlers::start_session),
        )
        .route(
            "/api/accounting/sessions/end",
            post(handlers::end_session),
        )
        .route("/api/agent/user-active", post(handlers::agent_user_active))
        .route("/api/agent/user-inactive", post(handlers::agent_user_inactive))
        .route("/api/firewall/interfaces", get(handlers::list_interfaces))
        .route("/api/firewall/interfaces/zone", post(handlers::set_interface_zone))
        .route(
            "/api/routing/routes",
            get(handlers::list_routes).post(handlers::add_route),
        )
        .route(
            "/api/routing/default",
            post(handlers::set_default_gateway),
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