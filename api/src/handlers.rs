use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};

use firewall::{
    FirewallService,
    Protocol,
    RuleAction,
    RuleDirection,
    Zone
};
use routing::RoutingService;
use accounting::{AccountingError, AccountingService};
use shared_kernel::prelude::*;

use crate::console::CONSOLE_HTML;

use crate::state::AppState;

use storage::{SessionRepo, UserRepo, AccountingRepo, AuditRepo, DbPool, IdentityRepo};
use crate::auth::{generate_token, AuthUser};
use uuid::Uuid;
use crate::dto::{
    AccountingSessionDto,
    CreateIdentityRequest,
    AgentUserActiveRequest,
    AgentUserInactiveRequest,
    IdentityDto,
    CreateUserRequest,
    UserDto,
    LoginRequest,
    LoginResponse,
    MeResponse,
    EndSessionRequest,
    StartSessionRequest,
    InterfaceDto,
    SetZoneRequest,
    AddRouteRequest,
    RouteDto,
    SetDefaultGatewayRequest,
    CommandDto,
    CoreServiceDto,
    CreateFirewallRuleRequest,
    ErrorDto,
    ExecuteCommandRequest,
    ExecuteCommandResponse,
    FirewallRuleDto,
    ModuleDto,
    HistoryQuery,
    UsageQuery,
    UpdateIdentityRequest
};

pub async fn console() -> Html<&'static str> {
    Html(CONSOLE_HTML)
}

pub async fn list_modules(_user: AuthUser, State(state): State<AppState>) -> Json<Vec<ModuleDto>> {
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

pub async fn list_commands(_user: AuthUser, State(state): State<AppState>) -> Json<Vec<CommandDto>> {
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

pub async fn list_core_services(_user: AuthUser) -> Json<Vec<CoreServiceDto>> {
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
    _user: AuthUser,
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

pub async fn list_firewall_rules(_user: AuthUser, State(state): State<AppState>) -> impl IntoResponse {
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
    _user: AuthUser,
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
    _user: AuthUser,
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

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    let users = UserRepo::new(pool.inner());
    let user = match users.find_by_username(&req.username).await {
        Ok(Some(u)) if u.enabled => u,
        Ok(_) => {
            let _ = AuditRepo::new(pool.inner())
                .log(
                    Some(&req.username),
                    "login_failed",
                    None,
                    serde_json::json!({ "reason": "invalid_credentials_or_disabled" }),
                )
                .await;

            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorDto {
                    error: "invalid username or password".into(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorDto {
                    error: e.to_string(),
                }),
            )
                .into_response();
        }
    };

    if !UserRepo::verify_password(&req.password, &user.password_hash) {
        let _ = AuditRepo::new(pool.inner())
            .log(
                Some(&req.username),
                "login_failed",
                None,
                serde_json::json!({ "reason": "wrong_password" }),
            )
            .await;

        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorDto {
                error: "invalid username or password".into(),
            }),
        )
            .into_response();
    }

    let token = generate_token();
    let sessions = SessionRepo::new(pool.inner());
    if let Err(e) = sessions.create(user.id, &token, 24, None, None).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response();
    }

    let _ = AuditRepo::new(pool.inner())
        .log(
            Some(&user.username),
            "login",
            None,
            serde_json::json!({ "role": user.role }),
        )
        .await;

    (
        StatusCode::OK,
        Json(LoginResponse {
            token,
            username: user.username,
            role: user.role,
            display_name: user.display_name,
        }),
    )
        .into_response()
}

pub async fn me(AuthUser(user): AuthUser) -> impl IntoResponse {
    Json(MeResponse {
        id: user.id.to_string(),
        username: user.username,
        role: user.role,
        display_name: user.display_name,
    })
}

pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let Some(token) = crate::auth::extract_bearer(&headers) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    if let Some(pool) = state.services.resolve::<DbPool>() {
        let sessions = SessionRepo::new(pool.inner());

        if let Ok(Some(user)) = sessions.find_user_by_token(&token).await {
            let _ = AuditRepo::new(pool.inner())
                .log(Some(&user.username), "logout", None, serde_json::json!({}))
                .await;
        }

        let _ = sessions.revoke(&token).await;
    }

    StatusCode::NO_CONTENT.into_response()
}

pub async fn list_users(
    user: AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if user.0.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorDto {
                error: "admin only".into(),
            }),
        )
            .into_response();
    }

    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    match UserRepo::new(pool.inner()).list().await {
        Ok(rows) => {
            let users: Vec<UserDto> = rows
                .into_iter()
                .map(|u| UserDto {
                    id: u.id.to_string(),
                    username: u.username,
                    role: u.role,
                    display_name: u.display_name,
                    enabled: u.enabled,
                })
                .collect();
            (StatusCode::OK, Json(users)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn create_user(
    user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    if user.0.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorDto {
                error: "admin only".into(),
            }),
        )
            .into_response();
    }

    let role = match req.role.as_str() {
        "admin" | "operator" | "viewer" => req.role.as_str(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorDto {
                    error: "role must be admin|operator|viewer".into(),
                }),
            )
                .into_response();
        }
    };

    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    match UserRepo::new(pool.inner())
        .create(
            &req.username,
            &req.password,
            req.display_name.as_deref(),
            role,
        )
        .await
    {
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

pub async fn list_identities(
    _user: AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    match IdentityRepo::new(pool.inner()).list().await {
        Ok(rows) => {
            let list: Vec<IdentityDto> = rows
                .into_iter()
                .map(|r| IdentityDto {
                    id: r.id.to_string(),
                    username: r.username,
                    display_name: r.display_name,
                    source: r.source,
                    enabled: r.enabled,
                })
                .collect();
            (StatusCode::OK, Json(list)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn create_identity(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateIdentityRequest>,
) -> impl IntoResponse {
    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    match IdentityRepo::new(pool.inner())
        .create(
            &req.username,
            req.display_name.as_deref(),
            &req.source,
        )
        .await
    {
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

pub async fn list_active_sessions(
    _user: AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    let fw = state.services.resolve::<FirewallService>().cloned();

    match AccountingRepo::new(pool.inner()).list_active().await {
        Ok(rows) => {
            let list: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    let (live_in, live_out, has_live) = match &fw {
                        Some(f) => match f.read_accounting(r.id) {
                            Ok((i, o)) => (i, o, true),
                            Err(_) => (r.bytes_in, r.bytes_out, false),
                        },
                        None => (r.bytes_in, r.bytes_out, false),
                    };
                    let hostname = r
                        .metadata
                        .get("hostname")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    serde_json::json!({
                        "id": r.id.to_string(),
                        "identity_id": r.identity_id.map(|i| i.to_string()),
                        "protocol": r.protocol,
                        "ip_address": r.ip_address,
                        "hostname": hostname,
                        "started_at": r.started_at.to_rfc3339(),
                        "ended_at": r.ended_at.map(|t| t.to_rfc3339()),
                        "bytes_in": r.bytes_in,
                        "bytes_out": r.bytes_out,
                        "live_bytes_in": live_in,
                        "live_bytes_out": live_out,
                        "has_live_counters": has_live,
                    })
                })
                .collect();
            (StatusCode::OK, Json(list)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn start_session(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<StartSessionRequest>,
) -> impl IntoResponse {
    let protocol = req.protocol.trim().to_lowercase();
    if protocol.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: "protocol is required (vpn|nac|portal|other)".into(),
            }),
        )
            .into_response();
    }

    let identity_id = match req.identity_id.as_deref() {
        None | Some("") => None,
        Some(s) => match Uuid::parse_str(s) {
            Ok(id) => Some(id),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorDto {
                        error: "invalid identity_id".into(),
                    }),
                )
                    .into_response();
            }
        },
    };

    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    match AccountingRepo::new(pool.inner())
        .start_session(identity_id, &protocol, req.ip_address.as_deref())
        .await
    {
        Ok(id) => {
            let _ = AuditRepo::new(pool.inner())
                .log(
                    identity_id.map(|i| i.to_string()).as_deref(),
                    "session_start",
                    Some(&id.to_string()),
                    serde_json::json!({ "protocol": protocol }),
                )
                .await;

            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "id": id.to_string() })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn end_session(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<EndSessionRequest>,
) -> impl IntoResponse {
    let Ok(session_id) = Uuid::parse_str(&req.session_id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: "invalid session_id".into(),
            }),
        )
            .into_response();
    };

    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    // ابتدا counterهای nft را بخوان و پاک کن تا در kernel نشت نکنند
    let (mut bytes_in, mut bytes_out) = (req.bytes_in, req.bytes_out);
    if let Some(fw) = state.services.resolve::<FirewallService>().cloned() {
        match fw.stop_accounting(session_id) {
            Ok((i, o)) => {
                bytes_in = i;
                bytes_out = o;
            }
            Err(e) => {
                eprintln!("[api] stop_accounting {session_id}: {e}");
            }
        }
    }

    let terminate_cause = req.terminate_cause.clone().unwrap_or_else(|| "manual".into());

    match AccountingRepo::new(pool.inner())
        .end_session(session_id, bytes_in, bytes_out, Some(&terminate_cause))
        .await
    {
        Ok(()) => {
            let _ = AuditRepo::new(pool.inner())
                .log(
                    None,
                    "session_end",
                    Some(&session_id.to_string()),
                    serde_json::json!({
                        "bytes_in": bytes_in,
                        "bytes_out": bytes_out,
                        "terminate_cause": terminate_cause,
                    }),
                )
                .await;

            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn list_interfaces(
    _user: AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(fw) = state.services.resolve::<FirewallService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "firewall unavailable".into(),
            }),
        )
            .into_response();
    };

    match fw.list_interfaces() {
        Ok(list) => {
            let dtos: Vec<InterfaceDto> = list
                .into_iter()
                .map(|i| InterfaceDto {
                    name: i.name,
                    zone: i.zone.to_string(),
                    up: i.up,
                    addresses: i.addresses,
                })
                .collect();
            (StatusCode::OK, Json(dtos)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn set_interface_zone(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<SetZoneRequest>,
) -> impl IntoResponse {
    let Some(zone) = Zone::parse(&req.zone) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: "zone must be lan|wan|dmz".into(),
            }),
        )
            .into_response();
    };

    if req.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: "name required".into(),
            }),
        )
            .into_response();
    }

    let Some(fw) = state.services.resolve::<FirewallService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "firewall unavailable".into(),
            }),
        )
            .into_response();
    };

    match fw.set_zone(req.name.trim(), zone) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn list_routes(
    _user: AuthUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let Some(rt) = state.services.resolve::<RoutingService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "routing unavailable".into(),
            }),
        )
            .into_response();
    };

    match rt.list_routes() {
        Ok(list) => {
            let dtos: Vec<RouteDto> = list
                .into_iter()
                .map(|r| RouteDto {
                    destination: r.destination,
                    gateway: r.gateway,
                    device: r.device,
                    proto: r.proto,
                    metric: r.metric,
                })
                .collect();
            (StatusCode::OK, Json(dtos)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn add_route(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AddRouteRequest>,
) -> impl IntoResponse {
    let Some(rt) = state.services.resolve::<RoutingService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "routing unavailable".into(),
            }),
        )
            .into_response();
    };

    match rt.add_route(
        &req.destination,
        req.gateway.as_deref(),
        req.device.as_deref(),
    ) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn set_default_gateway(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<SetDefaultGatewayRequest>,
) -> impl IntoResponse {
    let Some(rt) = state.services.resolve::<RoutingService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "routing unavailable".into(),
            }),
        )
            .into_response();
    };

    match rt.set_default_gateway(&req.gateway, req.device.as_deref()) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn agent_user_active(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AgentUserActiveRequest>,
) -> impl IntoResponse {
    let Some(acc) = state.services.resolve::<AccountingService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "accounting unavailable".into(),
            }),
        )
            .into_response();
    };

    match acc
        .user_active(
            &req.username,
            req.hostname.as_deref(),
            req.ip_address.as_deref(),
            req.os.as_deref(),      // ← این خط اضافه شد
        )
        .await
    {
        Ok(out) => {
            let status = if out.reused {
                StatusCode::OK
            } else {
                StatusCode::CREATED
            };
            (
                status,
                Json(serde_json::json!({
                    "session_id": out.session_id.to_string(),
                    "identity_id": out.identity_id.to_string(),
                    "reused": out.reused,
                    "accounting_error": out.accounting_error,
                })),
            )
                .into_response()
        }
        Err(AccountingError::IdentityDisabled) => (
            StatusCode::FORBIDDEN,
            Json(ErrorDto {
                error: "identity disabled".into(),
            }),
        )
            .into_response(),
        Err(AccountingError::DbUnavailable) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
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

pub async fn agent_user_inactive(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AgentUserInactiveRequest>,
) -> impl IntoResponse {
    let Some(acc) = state.services.resolve::<AccountingService>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "accounting unavailable".into(),
            }),
        )
            .into_response();
    };

    match acc
        .user_inactive(&req.username, req.bytes_in, req.bytes_out)
        .await
    {
        Ok(n) => (
            StatusCode::OK,
            Json(serde_json::json!({ "closed": n })),
        )
            .into_response(),
        Err(AccountingError::IdentityNotFound) => (
            StatusCode::NOT_FOUND,
            Json(ErrorDto {
                error: "identity not found".into(),
            }),
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

fn parse_dt_param(
    s: Option<&str>,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, axum::response::Response> {
    match s {
        None | Some("") => Ok(None),
        Some(v) => match chrono::DateTime::parse_from_rfc3339(v) {
            Ok(dt) => Ok(Some(dt.with_timezone(&chrono::Utc))),
            Err(_) => Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorDto {
                    error: format!("invalid datetime '{v}' (use RFC3339)"),
                }),
            )
                .into_response()),
        },
    }
}

/// GET /api/accounting/sessions/history?identity_id=&from=&to=&limit=&offset=
pub async fn list_session_history(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<HistoryQuery>,
) -> impl IntoResponse {
    let identity_id = match q.identity_id.as_deref() {
        None | Some("") => None,
        Some(s) => match Uuid::parse_str(s) {
            Ok(id) => Some(id),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorDto {
                        error: "invalid identity_id".into(),
                    }),
                )
                    .into_response()
            }
        },
    };

    let from = match parse_dt_param(q.from.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let to = match parse_dt_param(q.to.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let offset = q.offset.unwrap_or(0).max(0);

    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    match AccountingRepo::new(pool.inner())
        .list_history(identity_id, from, to, limit, offset)
        .await
    {
        Ok(rows) => {
            let list: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    let hostname = r
                        .metadata
                        .get("hostname")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    serde_json::json!({
                        "id": r.id.to_string(),
                        "identity_id": r.identity_id.map(|i| i.to_string()),
                        "protocol": r.protocol,
                        "ip_address": r.ip_address,
                        "hostname": hostname,
                        "started_at": r.started_at.to_rfc3339(),
                        "ended_at": r.ended_at.map(|t| t.to_rfc3339()),
                        "bytes_in": r.bytes_in,
                        "bytes_out": r.bytes_out,
                    })
                })
                .collect();
            (StatusCode::OK, Json(list)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// GET /api/accounting/usage?from=&to=
pub async fn usage_summary(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<UsageQuery>,
) -> impl IntoResponse {
    let from = match parse_dt_param(q.from.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let to = match parse_dt_param(q.to.as_deref()) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    match AccountingRepo::new(pool.inner()).usage_summary(from, to).await {
        Ok(rows) => {
            let list: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "identity_id": r.identity_id.map(|i| i.to_string()),
                        "username": r.username,
                        "sessions": r.sessions,
                        "bytes_in": r.bytes_in,
                        "bytes_out": r.bytes_out,
                    })
                })
                .collect();
            (StatusCode::OK, Json(list)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// PATCH /api/identities/{id}  { "enabled": false }
pub async fn update_identity(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateIdentityRequest>,
) -> impl IntoResponse {
    // فقط ادمین — اگر نام فیلد نقش در AuthUser شما فرق دارد، با الگوی create_user هماهنگ کن
    if user.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorDto {
                error: "admin only".into(),
            }),
        )
            .into_response();
    }

    let Some(pool) = state.services.resolve::<DbPool>() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorDto {
                error: "database unavailable".into(),
            }),
        )
            .into_response();
    };

    match IdentityRepo::new(pool.inner()).set_enabled(id, req.enabled).await {
        Ok(()) => {
            let _ = AuditRepo::new(pool.inner())
                .log(
                    None,
                    "identity_updated",
                    Some(&id.to_string()),
                    serde_json::json!({ "enabled": req.enabled }),
                )
                .await;
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorDto {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}