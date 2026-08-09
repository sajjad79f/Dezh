use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};

use shared_kernel::prelude::*;

use crate::dto::{
    CommandDto,
    CoreServiceDto,
    ErrorDto,
    ExecuteCommandRequest,
    ExecuteCommandResponse,
    ModuleDto,
};
use crate::state::AppState;
use crate::console::CONSOLE_HTML;

pub async fn console() -> Html<&'static str> {
    Html(CONSOLE_HTML)
}

pub async fn list_modules(
    State(state): State<AppState>,
) -> Json<Vec<ModuleDto>> {
    let modules = state
        .modules
        .descriptors()
        .into_iter()
        .map(|d| ModuleDto {
            id: d.id.to_string(),
            name: d.name.to_string(),
            version: d.version.to_string(),
        })
        .collect();

    Json(modules)
}

pub async fn list_commands(
    State(state): State<AppState>,
) -> Json<Vec<CommandDto>> {
    let commands = state
        .commands
        .commands()
        .iter()
        .map(|c| CommandDto {
            name: c.name().to_string(),
            description: c.description().to_string(),
        })
        .collect();

    Json(commands)
}

// Core Services are not yet tracked in a listable registry (they are
// registered into ServiceContainer by concrete type, see
// bootstrap::services::register). Until a CoreServiceRegistry exists
// (mirroring ModuleRegistry), this endpoint reports the fixed set that
// `bootstrap` always initializes at boot. Replace this once such a
// registry lands.
pub async fn list_core_services() -> Json<Vec<CoreServiceDto>> {
    let names = ["DEF", "DAI", "DKG", "DIE", "DDE"];

    let services = names
        .into_iter()
        .map(|name| CoreServiceDto {
            name: name.to_string(),
            status: "running".to_string(),
        })
        .collect();

    Json(services)
}

pub async fn execute_command(
    State(state): State<AppState>,
    Json(request): Json<ExecuteCommandRequest>,
) -> impl IntoResponse {
    let ctx = CommandContext::new(state.services.clone());

    let args: Vec<&str> = request.args.iter().map(String::as_str).collect();

    for command in state.commands.commands() {
        if command.name() == request.name {
            let output = command.execute(&ctx, &args);

            return (
                StatusCode::OK,
                Json(ExecuteCommandResponse { output }),
            )
                .into_response();
        }
    }

    (
        StatusCode::NOT_FOUND,
        Json(ErrorDto {
            error: format!("Unknown command: {}", request.name),
        }),
    )
        .into_response()
}
