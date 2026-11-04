mod console;
mod dto;
mod error;
mod handlers;
mod handlers_events;
mod rate_limit;
mod state;
mod auth;
mod validation;

pub use state::AppState;

use axum::http::{header, HeaderValue, Method};
use axum::routing::{delete, get, patch, post};
use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

/// CORS بر اساس whitelist از متغیر محیطی `DEZH_CORS_ORIGINS`
///
/// - ست شده (با کاما جدا شده): فقط آن origin ها مجازند
///   مثال: `DEZH_CORS_ORIGINS=http://192.168.1.106:5173,http://localhost:5173`
/// - ست نشده: cross-origin کاملاً بسته (پیش‌فرض امن).
///   Console داخلی و vite dev proxy همه same-origin هستند و تأثیری نمی‌بینند.
fn build_cors() -> CorsLayer {
    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
    ];
    let headers = [
        header::AUTHORIZATION,
        header::CONTENT_TYPE,
        header::ACCEPT,
    ];

    let origins_env = std::env::var("DEZH_CORS_ORIGINS").unwrap_or_default();
    let origins: Vec<HeaderValue> = origins_env
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();

    if origins.is_empty() {
        eprintln!(
            "[api] CORS: cross-origin requests disabled (set DEZH_CORS_ORIGINS to allow specific origins)"
        );
        // بدون allow_origin → همه preflight های cross-origin رد می‌شوند
        CorsLayer::new().allow_methods(methods).allow_headers(headers)
    } else {
        eprintln!(
            "[api] CORS: allowing {} configured origin(s)",
            origins.len()
        );
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(methods)
            .allow_headers(headers)
    }
}

pub fn router(state: AppState) -> Router {
    let cors = build_cors();

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
            "/api/firewall/rules/delete",
            post(handlers::delete_firewall_rule_by_body),
        )
        .route(
            "/api/firewall/rules/:id",
            delete(handlers::delete_firewall_rule).patch(handlers::update_firewall_rule),
        )
        .route(
            "/api/firewall/stats",
            get(handlers::firewall_rule_stats),
        )
        .route(
            "/api/events",
            get(handlers_events::list_events),
        )
        .route(
            "/api/events/summary",
            get(handlers_events::events_summary),
        )
        .route(
            "/api/firewall/connections",
            get(handlers::list_connections),
        )
        .route(
            "/api/firewall/sets",
            get(handlers::list_ip_sets).post(handlers::create_ip_set),
        )
        .route(
            "/api/firewall/sets/:name",
            delete(handlers::delete_ip_set).patch(handlers::update_ip_set),
        )
        .route(
            "/api/firewall/sets/:name/entries",
            post(handlers::add_set_entries).delete(handlers::remove_set_entry),
        )
        .route(
            "/api/firewall/settings",
            get(handlers::get_firewall_settings).put(handlers::update_firewall_settings),
        )
        .route(
            "/api/firewall/export",
            get(handlers::export_firewall_config),
        )
        .route(
            "/api/firewall/import",
            post(handlers::import_firewall_config),
        )
        .route(
            "/api/firewall/interfaces",
            get(handlers::list_interfaces),
        )
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/auth/password", post(handlers::change_password))
        .route("/api/users", get(handlers::list_users).post(handlers::create_user))
        .route(
            "/api/identities",
            get(handlers::list_identities).post(handlers::create_identity),
        )
        .route(
            "/api/identities/:id",
            patch(handlers::update_identity),
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
        .route(
            "/api/accounting/sessions/history",
            get(handlers::list_session_history),
        )
        .route("/api/accounting/usage", get(handlers::usage_summary))
        .route("/api/agent/user-active", post(handlers::agent_user_active))
        .route("/api/agent/user-inactive", post(handlers::agent_user_inactive))
        .route(
            "/api/firewall/nat",
            get(handlers::list_nat_rules).post(handlers::create_nat_rule),
        )
        .route(
            "/api/firewall/nat/:id",
            delete(handlers::delete_nat_rule),
        )
        .route(
            "/api/routing/routes",
            get(handlers::list_routes)
                .post(handlers::add_route)
                .delete(handlers::delete_route),
        )
        .route(
            "/api/routing/default",
            post(handlers::set_default_gateway),
        )
        .route(
            "/api/network/zones",
            get(handlers::list_zones).post(handlers::create_zone),
        )
        .route(
            "/api/network/zones/:name",
            patch(handlers::update_zone).delete(handlers::delete_zone),
        )
        .route(
            "/api/network/interfaces",
            get(handlers::list_network_interfaces),
        )
        .route(
            "/api/network/interfaces/zone",
            post(handlers::set_network_zone),
        )
        .route(
            "/api/network/interfaces/config",
            post(handlers::apply_interface_config),
        )
        .layer(cors)
        .with_state(state)
}

pub async fn serve(state: AppState, addr: &str) -> std::io::Result<()> {
    let app = router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Web Console listening on http://{addr}");
    // ConnectInfo لازم است تا rate limiter بتواند IP واقعی کلاینت را ببیند
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}