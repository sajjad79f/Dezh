use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};

use firewall::{FirewallService, Protocol, RuleAction, RuleDirection};
use shared_kernel::prelude::*;

use crate::console::CONSOLE_HTML;
use crate::dto::{
    CommandDto, CoreServiceDto, CreateFirewallRuleRequest, ErrorDto,
    ExecuteCommandRequest, ExecuteCommandResponse, FirewallRuleDto, ModuleDto,
};
use crate::state::AppState;

pub async fn console() -> Html<&'static str> {
    Html(CONSOLE_HTML)
}

pub async fn list_modules(State(state): State<AppState>) -> Json<Vec<ModuleDto>> {
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

pub async fn list_commands(State(state): State<AppState>) -> Json<Vec<CommandDto>> {
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
            return (StatusCode::OK, Json(ExecuteCommandResponse { output })).into_response();
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

// ── Firewall ──────────────────────────────────────────────

pub async fn list_firewall_rules(State(state): State<AppState>) -> impl IntoResponse {
    let Some(fw) = state.services.resolve::<FirewallService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "Firewall service not available".into(),
            }),
        )
            .into_response();
    };

    let rules: Vec<FirewallRuleDto> = fw
        .list_rules()
        .into_iter()
        .map(|r| FirewallRuleDto {
            id: r.id.to_string(),
            name: r.name,
            action: format!("{:?}", r.action).to_lowercase(),
            direction: format!("{:?}", r.direction).to_lowercase(),
            protocol: format!("{:?}", r.protocol).to_lowercase(),
            source: r.source,
            destination: r.destination,
            port: r.port,
            enabled: r.enabled,
            priority: r.priority,
        })
        .collect();

    (StatusCode::OK, Json(rules)).into_response()
}

pub async fn create_firewall_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateFirewallRuleRequest>,
) -> impl IntoResponse {
    let Some(fw) = state.services.resolve::<FirewallService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "Firewall service not available".into(),
            }),
        )
            .into_response();
    };

    let action = match req.action.as_str() {
        "allow" => RuleAction::Allow,
        "deny" => RuleAction::Deny,
        "drop" => RuleAction::Drop,
        "reject" => RuleAction::Reject,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorDto {
                    error: "Invalid action".into(),
                }),
            )
                .into_response();
        }
    };

    let direction = match req.direction.as_str() {
        "in" | "inbound" => RuleDirection::Inbound,
        "out" | "outbound" => RuleDirection::Outbound,
        "both" => RuleDirection::Both,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorDto {
                    error: "Invalid direction".into(),
                }),
            )
                .into_response();
        }
    };

    let protocol = match req.protocol.as_str() {
        "tcp" => Protocol::Tcp,
        "udp" => Protocol::Udp,
        "icmp" => Protocol::Icmp,
        "any" => Protocol::Any,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorDto {
                    error: "Invalid protocol".into(),
                }),
            )
                .into_response();
        }
    };

    match fw.add_rule(
        req.name,
        action,
        direction,
        protocol,
        req.source,
        req.destination,
        req.port,
        req.priority,
    ) {
        Ok(id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": id.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn delete_firewall_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(fw) = state.services.resolve::<FirewallService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "Firewall service not available".into(),
            }),
        )
            .into_response();
    };

    let Ok(uuid) = uuid::Uuid::parse_str(&id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: "Invalid rule id".into(),
            }),
        )
            .into_response();
    };

    match fw.remove_rule(uuid) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}