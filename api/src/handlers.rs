use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};

use std::net::SocketAddr;

use firewall::{
    ConnectionRow, FirewallRule, FirewallService, Protocol, RuleAction, RuleDirection,
    RuleSchedule,
};
use network::NetworkService;
use routing::RoutingService;
use accounting::{AccountingError, AccountingService};
use shared_kernel::prelude::*;

use crate::console::CONSOLE_HTML;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

use storage::{
    AccountingRepo, AuditRepo, DbPool, IdentityRepo, IpSetRow, NatRuleRow,
    SessionRepo, UserRepo,FqdnObjectRow
};
use crate::auth::{generate_token, AuthUser};
use crate::rate_limit::LoginRateLimiter;
use crate::validation;
use uuid::Uuid;
use crate::dto::{
    AddRouteRequest, AddSetEntriesRequest, AgentUserActiveRequest, AgentUserInactiveRequest,
    ChangePasswordRequest, CommandDto, ConnectionDto, CoreServiceDto, CreateFirewallRuleRequest,
    CreateIdentityRequest, CreateIpSetRequest, CreateNatRuleRequest, CreateUserRequest,
    CreateZoneRequest, DeleteFirewallRuleRequest, DeleteRouteRequest, EndSessionRequest,
    ExecuteCommandRequest, ExecuteCommandResponse, FirewallExport, FirewallRuleDto,
    FirewallSettingsDto, HistoryQuery, ImportResultDto, InterfaceConfigRequest, InterfaceDto,
    IpSetDto, LoginRequest, LoginResponse, MeResponse, ModuleDto, NatRuleDto,
    NetworkInterfaceDto, RemoveSetEntryRequest, RouteDto, RuleStatDto, SetDefaultGatewayRequest,
    SetNetworkZoneRequest, StartSessionRequest, UpdateFirewallRuleRequest,
    UpdateFirewallSettingsRequest, UpdateIdentityRequest, UpdateIpSetRequest, UpdateZoneRequest,
    UsageQuery, UserDto, ZoneDto,FqdnObjectDto, CreateFqdnRequest, UpdateFqdnRequest
};

// ─── تبدیل‌های مشترک (به‌جای تکرار در همه handlerها) ──────

fn parse_action(s: &str) -> Option<RuleAction> {
    match s {
        "allow" => Some(RuleAction::Allow),
        "deny" => Some(RuleAction::Deny),
        "drop" => Some(RuleAction::Drop),
        "reject" => Some(RuleAction::Reject),
        _ => None,
    }
}

fn parse_direction(s: &str) -> Option<RuleDirection> {
    match s {
        "in" | "inbound" => Some(RuleDirection::Inbound),
        "out" | "outbound" => Some(RuleDirection::Outbound),
        "both" => Some(RuleDirection::Both),
        _ => None,
    }
}

fn parse_protocol(s: &str) -> Option<Protocol> {
    match s {
        "tcp" => Some(Protocol::Tcp),
        "udp" => Some(Protocol::Udp),
        "icmp" => Some(Protocol::Icmp),
        "any" => Some(Protocol::Any),
        _ => None,
    }
}

fn parse_uuid(s: &str, what: &str) -> ApiResult<Uuid> {
    Uuid::parse_str(s.trim()).map_err(|_| ApiError::bad_request(format!("invalid {what} id")))
}

fn rule_to_dto(r: FirewallRule) -> FirewallRuleDto {
    // فاز ۴: وضعیت فعلی پنجره‌ی زمانی با ساعت سرور محاسبه می‌شود
    // (مرورگر کلاینت ملاک نیست)
    let (schedule_enabled, schedule_start, schedule_end, schedule_days, schedule_active) =
        match &r.schedule {
            Some(s) => (
                true,
                s.start.clone(),
                s.end.clone(),
                s.days.clone(),
                s.is_active_now(),
            ),
            None => (false, String::new(), String::new(), Vec::new(), false),
        };

    FirewallRuleDto {
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
        description: r.description,
        log: r.log,
        source_set: r.source_set,
        dest_set: r.dest_set,
        source_fqdn: r.source_fqdn,
        dest_fqdn: r.dest_fqdn,
        rate_limit: r.rate_limit,
        schedule_enabled,
        schedule_start,
        schedule_end,
        schedule_days,
        schedule_active,
    }
}

fn nat_to_dto(r: NatRuleRow) -> NatRuleDto {
    NatRuleDto {
        id: r.id.to_string(),
        name: r.name,
        enabled: r.enabled,
        kind: r.kind,
        interface: r.interface,
        source: r.source,
        destination: r.destination,
        protocol: r.protocol,
        dest_port: r.dest_port.map(|p| p as u16),
        target: r.target,
        description: r.description,
    }
}

fn set_to_dto(s: IpSetRow) -> IpSetDto {
    IpSetDto {
        id: s.id.to_string(),
        name: s.name,
        kind: s.kind,
        description: s.description,
        enabled: s.enabled,
        entries_count: s.entries.len(),
        entries: s.entries,
    }
}

fn conn_to_dto(c: ConnectionRow) -> ConnectionDto {
    ConnectionDto {
        protocol: c.protocol,
        state: c.state,
        src: c.src,
        dst: c.dst,
        sport: c.sport,
        dport: c.dport,
        timeout_secs: c.timeout_secs,
    }
}

/// ثبت audit در صورت اتصال DB — شکست آن هرگز جریان اصلی را نمی‌شکند
async fn audit(
    state: &AppState,
    actor: Option<&str>,
    action: &str,
    target: Option<&str>,
    details: serde_json::Value,
) {
    if let Some(pool) = state.services.resolve::<DbPool>() {
        let _ = AuditRepo::new(pool.inner())
            .log(actor, action, target, details)
            .await;
    }
}

// ─── عمومی ─────────────────────────────────────────────

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
) -> ApiResult<Response> {
    let ctx = CommandContext::new(state.services.clone());
    let args: Vec<&str> = request.args.iter().map(String::as_str).collect();

    for command in state.commands.commands() {
        if command.name() == request.name {
            let output = command.execute(&ctx, &args);
            return Ok((StatusCode::OK, Json(ExecuteCommandResponse { output })).into_response());
        }
    }

    Err(ApiError::bad_request(format!(
        "Unknown command: {}",
        request.name
    )))
}

// ─── Firewall: Rules ───────────────────────────────────

pub async fn list_firewall_rules(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<FirewallRuleDto>>> {
    let fw = state.service::<FirewallService>("firewall")?;

    Ok(Json(
        fw.list_rules().into_iter().map(rule_to_dto).collect(),
    ))
}

pub async fn create_firewall_rule(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateFirewallRuleRequest>,
) -> ApiResult<Response> {
    // ── اعتبارسنجی ورودی‌ها (قبل از دست زدن به سرویس)
    let name = validation::check_name(&req.name)
        .map_err(ApiError::bad_request)?
        .to_string();
    let source = validation::check_ip_or_cidr(&req.source).map_err(ApiError::bad_request)?;
    let destination =
        validation::check_ip_or_cidr(&req.destination).map_err(ApiError::bad_request)?;
    let port = validation::check_port(req.port).map_err(ApiError::bad_request)?;
    let priority = validation::check_priority(req.priority).map_err(ApiError::bad_request)?;
    let description = req.description.trim().to_string();
    // فاز ۳: سقف تطبیق قانون در ثانیه (1..100_000) — None = بی‌سقف
    let rate_limit = req.rate_limit.map(|v| v.clamp(1, 100_000));
    // فاز ۴: زمان‌بندی قانون (اختیاری)
    let schedule = build_schedule(
        req.schedule_enabled,
        &req.schedule_start,
        &req.schedule_end,
        &req.schedule_days,
    )?;

    let action = parse_action(&req.action)
        .ok_or_else(|| ApiError::bad_request("Invalid action"))?;
    let direction = parse_direction(&req.direction)
        .ok_or_else(|| ApiError::bad_request("Invalid direction"))?;
    let protocol = parse_protocol(&req.protocol)
        .ok_or_else(|| ApiError::bad_request("Invalid protocol"))?;

    let fw = state.service::<FirewallService>("firewall")?;

    // ارجاع به Address List باید واقعاً موجود باشد
    let source_set = validated_set_ref(fw, req.source_set.as_deref(), "source_set")?;
    let dest_set = validated_set_ref(fw, req.dest_set.as_deref(), "dest_set")?;
    let source_fqdn = validated_fqdn_ref(fw, req.source_fqdn.as_deref(), "source_fqdn")?;
    let dest_fqdn = validated_fqdn_ref(fw, req.dest_fqdn.as_deref(), "dest_fqdn")?;

    let rule = FirewallRule {
        description,
        log: req.log,
        source_set,
        dest_set,
        source_fqdn,
        dest_fqdn,
        rate_limit,
        schedule,
        ..FirewallRule::new(&name, action, direction, protocol, &source, &destination, port, priority)
    };

    let id = fw.add_rule_full(rule)?;

    audit(
        &state,
        None,
        "firewall_rule_created",
        Some(&id.to_string()),
        serde_json::json!({ "name": name }),
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id.to_string() })),
    )
        .into_response())
}

/// رشته ارجاع به Address List را تمیز و اعتبارسنجی می‌کند
fn validated_set_ref(
    fw: &FirewallService,
    value: Option<&str>,
    field: &str,
) -> ApiResult<Option<String>> {
    let Some(s) = value.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };

    let exists = fw
        .list_sets()
        .map(|sets| sets.iter().any(|x| x.name == s))
        .unwrap_or(false); // اگر DB نیست، سرویس موقع apply نادیده می‌گیرد

    if !exists {
        return Err(ApiError::bad_request(format!(
            "{field} '{s}' does not exist (create the address list first)"
        )));
    }
    Ok(Some(s.to_string()))
}

/// رشته ارجاع به شیء دامنه را تمیز و اعتبارسنجی می‌کند ("" = حذف ارجاع)
fn validated_fqdn_ref(
    fw: &FirewallService,
    value: Option<&str>,
    field: &str,
) -> ApiResult<Option<String>> {
    let Some(s) = value.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };

    let exists = fw
        .list_fqdn_objects()
        .map(|objs| objs.iter().any(|x| x.name == s))
        .unwrap_or(false); // اگر DB نیست، سرویس موقع apply نادیده می‌گیرد

    if !exists {
        return Err(ApiError::bad_request(format!(
            "{field} '{s}' does not exist (create the FQDN object first)"
        )));
    }
    Ok(Some(s.to_string()))
}

fn fqdn_to_dto(o: FqdnObjectRow) -> FqdnObjectDto {
    FqdnObjectDto {
        id: o.id.to_string(),
        name: o.name,
        fqdn: o.fqdn,
        description: o.description,
        enabled: o.enabled,
        resolved: o.resolved,
        last_resolved_at: o.last_resolved_at.map(|t| t.to_rfc3339()),
        last_error: o.last_error,
        created_at: o.created_at.to_rfc3339(),
        updated_at: o.updated_at.to_rfc3339(),
    }
}

/// فاز ۴: ساخت و اعتبارسنجی Schedule — خروجی None یعنی بدون زمان‌بندی
fn build_schedule(
    enabled: bool,
    start: &str,
    end: &str,
    days: &[String],
) -> ApiResult<Option<RuleSchedule>> {
    if !enabled {
        return Ok(None);
    }

    let start = start.trim();
    let end = end.trim();
    if start.is_empty() || end.is_empty() {
        return Err(ApiError::bad_request(
            "schedule_start and schedule_end are required when schedule is enabled",
        ));
    }

    // فرمت HH:MM با بازه‌ی معتبر
    let parse_hhmm = |v: &str| -> ApiResult<()> {
        let Some((h, m)) = v.split_once(':') else {
            return Err(ApiError::bad_request(format!(
                "invalid schedule time '{v}' (expected HH:MM)"
            )));
        };
        let (Ok(h), Ok(m)) = (h.trim().parse::<u8>(), m.trim().parse::<u8>()) else {
            return Err(ApiError::bad_request(format!(
                "invalid schedule time '{v}' (expected HH:MM)"
            )));
        };
        if h > 23 || m > 59 {
            return Err(ApiError::bad_request(format!(
                "schedule time out of range: '{v}'"
            )));
        }
        Ok(())
    };
    parse_hhmm(start)?;
    parse_hhmm(end)?;

    // روزها: sun..sat — لیست خالی یعنی همه‌ی روزها
    const VALID_DAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
    let mut clean = Vec::new();
    for d in days {
        let d = d.trim().to_lowercase();
        if d.is_empty() {
            continue;
        }
        if !VALID_DAYS.contains(&d.as_str()) {
            return Err(ApiError::bad_request(format!(
                "invalid schedule day '{d}' (expected sun..sat)"
            )));
        }
        if !clean.contains(&d) {
            clean.push(d);
        }
    }

    Ok(Some(RuleSchedule {
        start: start.to_string(),
        end: end.to_string(),
        days: clean,
    }))
}

/// PATCH /api/firewall/rules/:id — ویرایش کامل قانون (فاز ۴)
pub async fn update_firewall_rule(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFirewallRuleRequest>,
) -> ApiResult<StatusCode> {
    let uuid = parse_uuid(&id, "rule")?;

    if req.enabled.is_none()
        && req.priority.is_none()
        && req.name.is_none()
        && req.description.is_none()
        && req.log.is_none()
        && req.rate_limit.is_none()
        && req.action.is_none()
        && req.direction.is_none()
        && req.protocol.is_none()
        && req.source.is_none()
        && req.destination.is_none()
        && req.port.is_none()
        && req.source_set.is_none()
        && req.dest_set.is_none()
        && req.source_fqdn.is_none()
        && req.dest_fqdn.is_none()
        && req.schedule_enabled.is_none()
        && req.schedule_start.is_none()
        && req.schedule_end.is_none()
        && req.schedule_days.is_none()
    {
        return Err(ApiError::bad_request("nothing to update"));
    }

    // اعتبارسنجی مقادیر متنی قبل از اعمال
    let new_name = match &req.name {
        Some(v) => Some(
            validation::check_name(v)
                .map_err(ApiError::bad_request)?
                .to_string(),
        ),
        None => None,
    };
    let new_desc = match &req.description {
        Some(v) => Some(
            validation::check_description(v)
                .map_err(ApiError::bad_request)?
                .to_string(),
        ),
        None => None,
    };

    // ── فاز ۴: اعتبارسنجی فیلدهای تطبیق ──
    let new_action = match &req.action {
        Some(v) => Some(parse_action(v).ok_or_else(|| ApiError::bad_request("Invalid action"))?),
        None => None,
    };
    let new_direction = match &req.direction {
        Some(v) => {
            Some(parse_direction(v).ok_or_else(|| ApiError::bad_request("Invalid direction"))?)
        }
        None => None,
    };
    let new_protocol = match &req.protocol {
        Some(v) => Some(parse_protocol(v).ok_or_else(|| ApiError::bad_request("Invalid protocol"))?),
        None => None,
    };
    let new_source = match &req.source {
        Some(v) => Some(validation::check_ip_or_cidr(v).map_err(ApiError::bad_request)?),
        None => None,
    };
    let new_destination = match &req.destination {
        Some(v) => Some(validation::check_ip_or_cidr(v).map_err(ApiError::bad_request)?),
        None => None,
    };
    // 0 = حذف پورت (any)
    let new_port = match req.port {
        Some(0) => Some(None),
        Some(p) => Some(Some(p)),
        None => None,
    };

    let fw = state.service::<FirewallService>("firewall")?;

    // ارجاع به Address List باید واقعاً موجود باشد ("" یعنی حذف ارجاع)
    let source_set = validated_set_ref(fw, req.source_set.as_deref(), "source_set")?;
    let dest_set = validated_set_ref(fw, req.dest_set.as_deref(), "dest_set")?;
    let source_fqdn = validated_fqdn_ref(fw, req.source_fqdn.as_deref(), "source_fqdn")?;
    let dest_fqdn = validated_fqdn_ref(fw, req.dest_fqdn.as_deref(), "dest_fqdn")?;

    // ── فاز ۴: زمان‌بندی — قرارداد: هر ۴ فیلد باید با هم فرستاده شوند ──
    // هیچ‌کدام = دست نخوردن؛ هر ۴ با هم = اعمال (false یعنی حذف schedule)
    // ── فاز ۴: زمان‌بندی — هر ۴ فیلد باید با هم ارسال شوند ──
    let schedule = match (
        &req.schedule_enabled,
        &req.schedule_start,
        &req.schedule_end,
        &req.schedule_days,
    ) {
        (None, None, None, None) => None,
        (Some(enabled), Some(start), Some(end), Some(days)) => {
            Some(build_schedule(*enabled, start, end, days)?)
        }
        _ => {
            return Err(ApiError::bad_request(
                "schedule must be sent as all 4 fields together: schedule_enabled, schedule_start, schedule_end, schedule_days",
            ));
        }
    };

    fw.update_rule(uuid, |r| {
        if let Some(v) = req.enabled {
            r.enabled = v;
        }
        if let Some(v) = req.priority {
            r.priority = v.clamp(1, 100_000);
        }
        if let Some(v) = &new_name {
            r.name = v.clone();
        }
        if let Some(v) = &new_desc {
            r.description = v.clone();
        }
        if let Some(v) = req.log {
            r.log = v;
        }
        // فاز ۳: 0 یعنی حذف سقف
        if let Some(v) = req.rate_limit {
            r.rate_limit = if v == 0 { None } else { Some(v.clamp(1, 100_000)) };
        }
        // ── فاز ۴: فیلدهای تطبیق ──
        if let Some(v) = new_action {
            r.action = v;
        }
        if let Some(v) = new_direction {
            r.direction = v;
        }
        if let Some(v) = new_protocol {
            r.protocol = v;
        }
        if let Some(v) = &new_source {
            r.source = v.clone();
        }
        if let Some(v) = &new_destination {
            r.destination = v.clone();
        }
        if let Some(v) = new_port {
            r.port = v;
        }
        // فقط اگر کلاینت صراحتاً فیلد را فرستاده باشد دست می‌زنیم
        if req.source_set.is_some() {
            r.source_set = source_set.clone();
        }
        if req.dest_set.is_some() {
            r.dest_set = dest_set.clone();
        }
        if req.source_fqdn.is_some() {
            r.source_fqdn = source_fqdn.clone();
        }
        if req.dest_fqdn.is_some() {
            r.dest_fqdn = dest_fqdn.clone();
        }
        // فاز ۴: زمان‌بندی — فقط اگر هر ۴ فیلد با هم آمده باشند
        if let Some(v) = &schedule {
            r.schedule = v.clone();
        }
    })?;

    audit(&state, None, "firewall_rule_updated", Some(&id), serde_json::json!({
        "enabled": req.enabled,
        "priority": req.priority,
        "schedule_updated": schedule.is_some(),
    }))
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_firewall_rule(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let uuid = parse_uuid(&id, "rule")?;

    let fw = state.service::<FirewallService>("firewall")?;
    fw.remove_rule(uuid)?;

    audit(&state, None, "firewall_rule_deleted", Some(&id), serde_json::json!({})).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_firewall_rule_by_body(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<DeleteFirewallRuleRequest>,
) -> ApiResult<StatusCode> {
    let uuid = parse_uuid(&req.id, "rule")?;

    let fw = state.service::<FirewallService>("firewall")?;
    fw.remove_rule(uuid)?;

    audit(&state, None, "firewall_rule_deleted", Some(&req.id), serde_json::json!({})).await;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/firewall/stats — آمار per-rule از nftables counter
pub async fn firewall_rule_stats(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<RuleStatDto>>> {
    let fw = state.service::<FirewallService>("firewall")?;

    let list = fw
        .rule_stats()?
        .into_iter()
        .map(|(id, (packets, bytes))| RuleStatDto {
            rule_id: id.to_string(),
            packets,
            bytes,
        })
        .collect();

    Ok(Json(list))
}

/// GET /api/firewall/connections — اتصال‌های زنده (conntrack)
pub async fn list_connections(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<ConnectionDto>>> {
    let fw = state.service::<FirewallService>("firewall")?;

    Ok(Json(
        fw.list_connections(500)?
            .into_iter()
            .map(conn_to_dto)
            .collect(),
    ))
}

// ─── Firewall: Address Lists (NGFW) ────────────────────

pub async fn list_ip_sets(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<IpSetDto>>> {
    let fw = state.service::<FirewallService>("firewall")?;

    Ok(Json(fw.list_sets()?.into_iter().map(set_to_dto).collect()))
}

pub async fn create_ip_set(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateIpSetRequest>,
) -> ApiResult<Response> {
    let description =
        validation::check_description(&req.description)
            .map_err(ApiError::bad_request)?
            .to_string();

    let fw = state.service::<FirewallService>("firewall")?;
    let id = fw.create_set(&req.name, &req.kind, &description, req.enabled)?;

    audit(
        &state,
        None,
        "ipset_created",
        Some(&req.name),
        serde_json::json!({ "kind": req.kind }),
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id.to_string() })),
    )
        .into_response())
}

pub async fn delete_ip_set(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let fw = state.service::<FirewallService>("firewall")?;
    fw.delete_set(&name)?;

    audit(&state, None, "ipset_deleted", Some(&name), serde_json::json!({})).await;
    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/firewall/sets/:name — فعال/غیرفعال کردن لیست
pub async fn update_ip_set(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<UpdateIpSetRequest>,
) -> ApiResult<StatusCode> {
    let fw = state.service::<FirewallService>("firewall")?;
    fw.set_set_enabled(&name, req.enabled)?;

    audit(
        &state,
        None,
        "ipset_updated",
        Some(&name),
        serde_json::json!({ "enabled": req.enabled }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

// ─── فاز ۵: اشیاء FQDN ─────────────────────────────────────

/// GET /api/firewall/fqdn
pub async fn list_fqdn_objects(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<FqdnObjectDto>>> {
    let fw = state.service::<FirewallService>("firewall")?;
    Ok(Json(fw.list_fqdn_objects()?.into_iter().map(fqdn_to_dto).collect()))
}

/// POST /api/firewall/fqdn
pub async fn create_fqdn_object(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateFqdnRequest>,
) -> ApiResult<Response> {
    let description = validation::check_description(&req.description)
        .map_err(ApiError::bad_request)?
        .to_string();

    let fw = state.service::<FirewallService>("firewall")?;
    let id = fw.create_fqdn_object(&req.name, &req.fqdn, &description, req.enabled)?;

    audit(
        &state,
        None,
        "fqdn_object_created",
        Some(&req.name),
        serde_json::json!({ "fqdn": req.fqdn }),
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id.to_string() })),
    )
        .into_response())
}

/// DELETE /api/firewall/fqdn/:name
pub async fn delete_fqdn_object(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let fw = state.service::<FirewallService>("firewall")?;
    fw.delete_fqdn_object(&name)?;

    audit(&state, None, "fqdn_object_deleted", Some(&name), serde_json::json!({})).await;
    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/firewall/fqdn/:name — فعال/غیرفعال کردن شیء
pub async fn update_fqdn_object(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<UpdateFqdnRequest>,
) -> ApiResult<StatusCode> {
    let fw = state.service::<FirewallService>("firewall")?;
    fw.set_fqdn_enabled(&name, req.enabled)?;

    audit(
        &state,
        None,
        "fqdn_object_updated",
        Some(&name),
        serde_json::json!({ "enabled": req.enabled }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/firewall/sets/:name/entries — افزودن گروهی (IP/CIDR)
pub async fn add_set_entries(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<AddSetEntriesRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let fw = state.service::<FirewallService>("firewall")?;
    let added = fw.add_set_entries(&name, req.entries)?;

    audit(
        &state,
        None,
        "ipset_entries_added",
        Some(&name),
        serde_json::json!({ "added": added }),
    )
    .await;
    Ok(Json(serde_json::json!({ "added": added })))
}

/// DELETE /api/firewall/sets/:name/entries — حذف یک ورودی
pub async fn remove_set_entry(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<RemoveSetEntryRequest>,
) -> ApiResult<StatusCode> {
    let fw = state.service::<FirewallService>("firewall")?;
    fw.remove_set_entry(&name, &req.cidr)?;

    audit(
        &state,
        None,
        "ipset_entry_removed",
        Some(&name),
        serde_json::json!({ "cidr": req.cidr }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Firewall: Settings (DoS protection) ───────────────

pub async fn get_firewall_settings(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<FirewallSettingsDto>> {
    let fw = state.service::<FirewallService>("firewall")?;
    let s = fw.get_settings();

    Ok(Json(FirewallSettingsDto {
        dos_protection: s.dos_protection,
        syn_rate: s.syn_rate,
    }))
}

pub async fn update_firewall_settings(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateFirewallSettingsRequest>,
) -> ApiResult<StatusCode> {
    let fw = state.service::<FirewallService>("firewall")?;
    let current = fw.get_settings();

    let dos_protection = req.dos_protection.unwrap_or(current.dos_protection);
    let syn_rate = req.syn_rate.unwrap_or(current.syn_rate).clamp(1, 100_000);

    fw.set_settings(dos_protection, syn_rate)?;

    audit(
        &state,
        None,
        "firewall_settings_updated",
        None,
        serde_json::json!({ "dos_protection": dos_protection, "syn_rate": syn_rate }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Firewall: Import / Export ─────────────────────────

/// GET /api/firewall/export — خروجی کامل پیکربندی فایروال (JSON)
pub async fn export_firewall_config(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<FirewallExport>> {
    let fw = state.service::<FirewallService>("firewall")?;

    let rules: Vec<FirewallRuleDto> = fw.list_rules().into_iter().map(rule_to_dto).collect();
    let nat_rules: Vec<NatRuleDto> = fw
        .list_nat_rules()
        .unwrap_or_default()
        .into_iter()
        .map(nat_to_dto)
        .collect();
    let ip_sets: Vec<IpSetDto> = fw
        .list_sets()
        .unwrap_or_default()
        .into_iter()
        .map(set_to_dto)
        .collect();

    Ok(Json(FirewallExport {
        version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        rules,
        nat_rules,
        ip_sets,
    }))
}

/// POST /api/firewall/import — ورود پیکربندی؛ تکراری‌ها (هم‌نام) رد می‌شوند
pub async fn import_firewall_config(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<FirewallExport>,
) -> ApiResult<Json<ImportResultDto>> {
    let fw = state.service::<FirewallService>("firewall")?;

    // ── قوانین فیلتر
    let existing_rules: std::collections::HashSet<String> =
        fw.list_rules().into_iter().map(|r| r.name).collect();
    let mut rules_added = 0usize;
    let mut rules_skipped = 0usize;

    for r in req.rules {
        let name = r.name.trim().to_string();
        if name.is_empty() || existing_rules.contains(&name) {
            rules_skipped += 1;
            continue;
        }

        let name = validation::check_name(&name).map_err(ApiError::bad_request)?.to_string();
        let source =
            validation::check_ip_or_cidr(&r.source).map_err(ApiError::bad_request)?;
        let destination =
            validation::check_ip_or_cidr(&r.destination).map_err(ApiError::bad_request)?;
        let port = validation::check_port(r.port).map_err(ApiError::bad_request)?;
        let priority =
            validation::check_priority(r.priority).map_err(ApiError::bad_request)?;

        let action = parse_action(&r.action)
            .ok_or_else(|| ApiError::bad_request(format!("invalid action '{}'", r.action)))?;
        let direction = parse_direction(&r.direction).ok_or_else(|| {
            ApiError::bad_request(format!("invalid direction '{}'", r.direction))
        })?;
        let protocol = parse_protocol(&r.protocol).ok_or_else(|| {
            ApiError::bad_request(format!("invalid protocol '{}'", r.protocol))
        })?;

        // فاز ۴: زمان‌بندی + سقف نرخ از فایل export حفظ می‌شوند
        let schedule = build_schedule(
            r.schedule_enabled,
            &r.schedule_start,
            &r.schedule_end,
            &r.schedule_days,
        )?;

        let rule = FirewallRule {
            description: r.description.clone(),
            log: r.log,
            source_set: None,
            dest_set: None,
            rate_limit: r.rate_limit.map(|v| v.clamp(1, 100_000)),
            schedule,
            ..FirewallRule::new(&name, action, direction, protocol, &source, &destination, port, priority)
        };
        fw.add_rule_full(rule)?;
        rules_added += 1;
    }

    // ── NAT
    let existing_nat: std::collections::HashSet<String> = fw
        .list_nat_rules()
        .unwrap_or_default()
        .into_iter()
        .map(|n| n.name)
        .collect();
    let mut nat_added = 0usize;
    let mut nat_skipped = 0usize;

    for n in req.nat_rules {
        let name = n.name.trim().to_string();
        if name.is_empty() || existing_nat.contains(&name) {
            nat_skipped += 1;
            continue;
        }

        let name = validation::check_name(&name).map_err(ApiError::bad_request)?.to_string();
        let interface = validation::check_iface_name(&n.interface)
            .map_err(ApiError::bad_request)?
            .to_string();
        let source = validation::check_ip_or_cidr(&n.source).map_err(ApiError::bad_request)?;
        let destination =
            validation::check_ip_or_cidr(&n.destination).map_err(ApiError::bad_request)?;
        let dest_port = validation::check_port(n.dest_port).map_err(ApiError::bad_request)?;
        let description = validation::check_description(&n.description)
            .map_err(ApiError::bad_request)?
            .to_string();

        let target = match n.target.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
            Some(t) => Some(validation::check_ip(t).map_err(ApiError::bad_request)?),
            None => None,
        };

        fw.add_nat_rule(
            &name,
            &n.kind,
            &interface,
            &source,
            &destination,
            &n.protocol,
            dest_port,
            target.as_deref(),
            &description,
        )?;
        nat_added += 1;
    }

    // ── Address Lists
    let existing_sets: std::collections::HashSet<String> = fw
        .list_sets()
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.name)
        .collect();
    let mut sets_added = 0usize;
    let mut sets_skipped = 0usize;

    for s in req.ip_sets {
        let name = s.name.trim().to_lowercase();
        if name.is_empty() || existing_sets.contains(&name) {
            sets_skipped += 1;
            continue;
        }

        fw.create_set(&name, &s.kind, &s.description, s.enabled)?;
        if !s.entries.is_empty() {
            fw.add_set_entries(&name, s.entries.clone())?;
        }
        sets_added += 1;
    }

    audit(
        &state,
        None,
        "firewall_imported",
        None,
        serde_json::json!({
            "rules_added": rules_added,
            "nat_added": nat_added,
            "sets_added": sets_added,
        }),
    )
    .await;

    Ok(Json(ImportResultDto {
        rules_added,
        rules_skipped,
        nat_added,
        nat_skipped,
        sets_added,
        sets_skipped,
    }))
}

// ─── Auth ──────────────────────────────────────────────

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Response> {
    let ip = addr.ip();

    // ── Rate limit: بعد از ۵ تلاش ناموفق در ۶۰ ثانیه، IP مسدود می‌شود
    if LoginRateLimiter::global().is_blocked(ip) {
        audit(
            &state,
            Some(&req.username),
            "login_rate_limited",
            None,
            serde_json::json!({ "ip": ip.to_string() }),
        )
        .await;

        return Err(ApiError::TooManyRequests {
            retry_after_secs: 60,
            message: "too many failed login attempts. try again in 60 seconds".into(),
        });
    }

    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let users = UserRepo::new(pool.inner());
    let user = match users.find_by_username(&req.username).await {
        Ok(Some(u)) if u.enabled => u,
        Ok(_) => {
            LoginRateLimiter::global().record_failure(ip);

            audit(
                &state,
                Some(&req.username),
                "login_failed",
                None,
                serde_json::json!({
                    "reason": "invalid_credentials_or_disabled",
                    "ip": ip.to_string(),
                }),
            )
            .await;

            return Err(ApiError::unauthorized("invalid username or password"));
        }
        Err(e) => return Err(ApiError::internal(e.to_string())),
    };

    if !UserRepo::verify_password(&req.password, &user.password_hash) {
        LoginRateLimiter::global().record_failure(ip);

        audit(
            &state,
            Some(&req.username),
            "login_failed",
            None,
            serde_json::json!({
                "reason": "wrong_password",
                "ip": ip.to_string(),
            }),
        )
        .await;

        return Err(ApiError::unauthorized("invalid username or password"));
    }

    let token = generate_token();
    LoginRateLimiter::global().clear(ip);

    let sessions = SessionRepo::new(pool.inner());
    sessions
        .create(user.id, &token, 24, Some(&ip.to_string()), None)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    audit(
        &state,
        Some(&user.username),
        "login",
        None,
        serde_json::json!({ "role": user.role }),
    )
    .await;

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            token,
            username: user.username,
            role: user.role,
            display_name: user.display_name,
            must_change_password: user.must_change_password,
        }),
    )
        .into_response())
}

pub async fn me(AuthUser(user): AuthUser) -> Json<MeResponse> {
    Json(MeResponse {
        id: user.id.to_string(),
        username: user.username,
        role: user.role,
        display_name: user.display_name,
        must_change_password: user.must_change_password,
    })
}

pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> ApiResult<StatusCode> {
    let Some(token) = crate::auth::extract_bearer(&headers) else {
        return Ok(StatusCode::NO_CONTENT);
    };

    if let Some(pool) = state.services.resolve::<DbPool>() {
        let sessions = SessionRepo::new(pool.inner());

        if let Ok(Some(user)) = sessions.find_user_by_token(&token).await {
            audit(&state, Some(&user.username), "logout", None, serde_json::json!({})).await;
        }

        let _ = sessions.revoke(&token).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/auth/password — تغییر رمز عبور کاربر جاری
///
/// رمز فعلی تأیید می‌شود، رمز جدید با سیاست رمز سنجیده می‌شود،
/// پرچم must_change_password صفر و همه‌ی sessionهای دیگر (به‌جز
/// همین مرورگر) باطل می‌شوند.
pub async fn change_password(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> ApiResult<StatusCode> {
    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    // ۱) رمز فعلی باید درست باشد
    if !UserRepo::verify_password(&req.current_password, &user.password_hash) {
        audit(
            &state,
            Some(&user.username),
            "password_change_failed",
            None,
            serde_json::json!({ "reason": "wrong_current_password" }),
        )
        .await;

        return Err(ApiError::unauthorized("current password is incorrect"));
    }

    // ۲) سیاست رمز جدید
    validation::check_password(&req.new_password).map_err(ApiError::bad_request)?;

    if req.new_password == req.current_password {
        return Err(ApiError::bad_request(
            "new password must be different from the current one",
        ));
    }

    // ۳) هش و ذخیره — update_password پرچم must_change_password را هم صفر می‌کند
    let new_hash = UserRepo::hash_password(&req.new_password)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    UserRepo::new(pool.inner())
        .update_password(user.id, &new_hash)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // ۴) باطل کردن بقیه sessionها — فقط session جاری زنده می‌ماند
    let sessions = SessionRepo::new(pool.inner());
    let current_token = crate::auth::extract_bearer(&headers);
    if let Err(e) = sessions
        .revoke_all_for_user(user.id, current_token.as_deref())
        .await
    {
        eprintln!("[api] revoke other sessions after password change failed: {e}");
    }

    audit(
        &state,
        Some(&user.username),
        "password_changed",
        None,
        serde_json::json!({}),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Users / Identities ────────────────────────────────

pub async fn list_users(
    user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<UserDto>>> {
    if user.0.role != "admin" {
        return Err(ApiError::forbidden("admin only"));
    }

    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let users = UserRepo::new(pool.inner())
        .list()
        .await?
        .into_iter()
        .map(|u| UserDto {
            id: u.id.to_string(),
            username: u.username,
            role: u.role,
            display_name: u.display_name,
            enabled: u.enabled,
            must_change_password: u.must_change_password,
        })
        .collect();

    Ok(Json(users))
}

pub async fn create_user(
    user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> ApiResult<Response> {
    if user.0.role != "admin" {
        return Err(ApiError::forbidden("admin only"));
    }

    let role = match req.role.as_str() {
        "admin" | "operator" | "viewer" => req.role.as_str(),
        _ => return Err(ApiError::bad_request("role must be admin|operator|viewer")),
    };

    validation::check_username(&req.username).map_err(ApiError::bad_request)?;
    validation::check_password(&req.password).map_err(ApiError::bad_request)?;
    if let Some(dn) = req.display_name.as_deref() {
        validation::check_description(dn).map_err(ApiError::bad_request)?;
    }

    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let id = UserRepo::new(pool.inner())
        .create(
            &req.username,
            &req.password,
            req.display_name.as_deref(),
            role,
        )
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id.to_string() })),
    )
        .into_response())
}

pub async fn list_identities(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<crate::dto::IdentityDto>>> {
    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let list = IdentityRepo::new(pool.inner())
        .list()
        .await?
        .into_iter()
        .map(|r| crate::dto::IdentityDto {
            id: r.id.to_string(),
            username: r.username,
            display_name: r.display_name,
            source: r.source,
            enabled: r.enabled,
        })
        .collect();

    Ok(Json(list))
}

pub async fn create_identity(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateIdentityRequest>,
) -> ApiResult<Response> {
    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let id = IdentityRepo::new(pool.inner())
        .create(&req.username, req.display_name.as_deref(), &req.source)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id.to_string() })),
    )
        .into_response())
}

pub async fn update_identity(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateIdentityRequest>,
) -> ApiResult<StatusCode> {
    if user.0.role != "admin" {
        return Err(ApiError::forbidden("admin only"));
    }

    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    IdentityRepo::new(pool.inner())
        .set_enabled(id, req.enabled)
        .await?;

    audit(
        &state,
        None,
        "identity_updated",
        Some(&id.to_string()),
        serde_json::json!({ "enabled": req.enabled }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Accounting ────────────────────────────────────────

pub async fn list_active_sessions(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    // فایروال اختیاری است — اگر نبود، مقادیر DB برمی‌گردد
    let fw = state.services.resolve::<FirewallService>().cloned();

    let list = AccountingRepo::new(pool.inner())
        .list_active()
        .await?
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

    Ok(Json(list))
}

pub async fn start_session(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<StartSessionRequest>,
) -> ApiResult<Response> {
    let protocol = req.protocol.trim().to_lowercase();
    if protocol.is_empty() {
        return Err(ApiError::bad_request(
            "protocol is required (vpn|nac|portal|other)",
        ));
    }

    let identity_id = match req.identity_id.as_deref() {
        None | Some("") => None,
        Some(s) => Some(parse_uuid(s, "identity")?),
    };

    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let id = AccountingRepo::new(pool.inner())
        .start_session(identity_id, &protocol, req.ip_address.as_deref())
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    audit(
        &state,
        identity_id.map(|i| i.to_string()).as_deref(),
        "session_start",
        Some(&id.to_string()),
        serde_json::json!({ "protocol": protocol }),
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id.to_string() })),
    )
        .into_response())
}

pub async fn end_session(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<EndSessionRequest>,
) -> ApiResult<StatusCode> {
    let session_id = parse_uuid(&req.session_id, "session")?;

    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

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

    let terminate_cause = req
        .terminate_cause
        .clone()
        .unwrap_or_else(|| "manual".into());

    AccountingRepo::new(pool.inner())
        .end_session(session_id, bytes_in, bytes_out, Some(&terminate_cause))
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    audit(
        &state,
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

    Ok(StatusCode::NO_CONTENT)
}

fn parse_dt_param(s: Option<&str>) -> ApiResult<Option<chrono::DateTime<chrono::Utc>>> {
    match s {
        None | Some("") => Ok(None),
        Some(v) => chrono::DateTime::parse_from_rfc3339(v)
            .map(|dt| Some(dt.with_timezone(&chrono::Utc)))
            .map_err(|_| ApiError::bad_request(format!("invalid datetime '{v}' (use RFC3339)"))),
    }
}

/// GET /api/accounting/sessions/history?identity_id=&from=&to=&limit=&offset=
pub async fn list_session_history(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<HistoryQuery>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let identity_id = match q.identity_id.as_deref() {
        None | Some("") => None,
        Some(s) => Some(parse_uuid(s, "identity")?),
    };

    let from = parse_dt_param(q.from.as_deref())?;
    let to = parse_dt_param(q.to.as_deref())?;

    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let offset = q.offset.unwrap_or(0).max(0);

    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let list = AccountingRepo::new(pool.inner())
        .list_history(identity_id, from, to, limit, offset)
        .await?
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

    Ok(Json(list))
}

/// GET /api/accounting/usage?from=&to=
pub async fn usage_summary(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<UsageQuery>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let from = parse_dt_param(q.from.as_deref())?;
    let to = parse_dt_param(q.to.as_deref())?;

    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let list = AccountingRepo::new(pool.inner())
        .usage_summary(from, to)
        .await?
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

    Ok(Json(list))
}

pub async fn agent_user_active(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AgentUserActiveRequest>,
) -> ApiResult<Response> {
    let acc = state.service::<AccountingService>("accounting")?;

    match acc
        .user_active(
            &req.username,
            req.hostname.as_deref(),
            req.ip_address.as_deref(),
            req.os.as_deref(),
        )
        .await
    {
        Ok(out) => {
            let status = if out.reused {
                StatusCode::OK
            } else {
                StatusCode::CREATED
            };
            Ok((
                status,
                Json(serde_json::json!({
                    "session_id": out.session_id.to_string(),
                    "identity_id": out.identity_id.to_string(),
                    "reused": out.reused,
                    "accounting_error": out.accounting_error,
                })),
            )
                .into_response())
        }
        Err(AccountingError::IdentityDisabled) => {
            Err(ApiError::forbidden("identity disabled"))
        }
        Err(AccountingError::DbUnavailable) => {
            Err(ApiError::service_unavailable("database unavailable"))
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

pub async fn agent_user_inactive(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AgentUserInactiveRequest>,
) -> ApiResult<Response> {
    let acc = state.service::<AccountingService>("accounting")?;

    match acc
        .user_inactive(&req.username, req.bytes_in, req.bytes_out)
        .await
    {
        Ok(n) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "closed": n })),
        )
            .into_response()),
        Err(AccountingError::IdentityNotFound) => {
            Err(ApiError::not_found("identity not found"))
        }
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

// ─── Interfaces / Routing ──────────────────────────────

pub async fn list_interfaces(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<InterfaceDto>>> {
    let fw = state.service::<FirewallService>("firewall")?;

    let dtos = fw
        .list_interfaces()?
        .into_iter()
        .map(|i| InterfaceDto {
            name: i.name,
            zone: i.zone.to_string(),
            up: i.up,
            addresses: i.addresses,
        })
        .collect();

    Ok(Json(dtos))
}

pub async fn list_routes(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<RouteDto>>> {
    let rt = state.service::<RoutingService>("routing")?;

    let dtos = rt
        .list_routes()?
        .into_iter()
        .map(|r| RouteDto {
            destination: r.destination,
            gateway: r.gateway,
            device: r.device,
            proto: r.proto,
            metric: r.metric,
        })
        .collect();

    Ok(Json(dtos))
}

pub async fn add_route(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<AddRouteRequest>,
) -> ApiResult<StatusCode> {
    // ── اعتبارسنجی ورودی‌ها
    let destination = validation::check_cidr(&req.destination).map_err(ApiError::bad_request)?;
    let gateway = match req.gateway.as_deref() {
        None => None,
        Some(g) if g.trim().is_empty() => None,
        Some(g) => Some(validation::check_ip(g).map_err(ApiError::bad_request)?),
    };
    let device = match req.device.as_deref() {
        None => None,
        Some(d) if d.trim().is_empty() => None,
        Some(d) => Some(
            validation::check_iface_name(d)
                .map_err(ApiError::bad_request)?
                .to_string(),
        ),
    };

    let rt = state.service::<RoutingService>("routing")?;
    rt.add_route(&destination, gateway.as_deref(), device.as_deref())?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_route(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<DeleteRouteRequest>,
) -> ApiResult<StatusCode> {
    let rt = state.service::<RoutingService>("routing")?;
    rt.delete_route(&req.destination, req.gateway.as_deref(), req.device.as_deref())?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default_gateway(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<SetDefaultGatewayRequest>,
) -> ApiResult<StatusCode> {
    let gateway = validation::check_ip(&req.gateway).map_err(ApiError::bad_request)?;
    let device = match req.device.as_deref() {
        None => None,
        Some(d) if d.trim().is_empty() => None,
        Some(d) => Some(
            validation::check_iface_name(d)
                .map_err(ApiError::bad_request)?
                .to_string(),
        ),
    };

    let rt = state.service::<RoutingService>("routing")?;
    rt.set_default_gateway(&gateway, device.as_deref())?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── NAT ───────────────────────────────────────────────

pub async fn list_nat_rules(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<NatRuleDto>>> {
    let fw = state.service::<FirewallService>("firewall")?;

    Ok(Json(
        fw.list_nat_rules()?.into_iter().map(nat_to_dto).collect(),
    ))
}

pub async fn create_nat_rule(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateNatRuleRequest>,
) -> ApiResult<Response> {
    // ── اعتبارسنجی ورودی‌ها
    let name = validation::check_name(&req.name)
        .map_err(ApiError::bad_request)?
        .to_string();
    let interface = validation::check_iface_name(&req.interface)
        .map_err(ApiError::bad_request)?
        .to_string();
    let source = validation::check_ip_or_cidr(&req.source).map_err(ApiError::bad_request)?;
    let destination =
        validation::check_ip_or_cidr(&req.destination).map_err(ApiError::bad_request)?;
    let dest_port = validation::check_port(req.dest_port).map_err(ApiError::bad_request)?;
    let description = validation::check_description(&req.description)
        .map_err(ApiError::bad_request)?
        .to_string();

    // kind و target با هم: dnat حتماً target (IP) لازم دارد
    let kind = req.kind.trim().to_ascii_lowercase();
    let target = match kind.as_str() {
        "masquerade" => {
            if req
                .target
                .as_deref()
                .map(str::trim)
                .is_some_and(|t| !t.is_empty())
            {
                return Err(ApiError::bad_request(
                    "masquerade rule must not have a target",
                ));
            }
            None
        }
        "dnat" => match req.target.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
            Some(t) => Some(validation::check_ip(t).map_err(ApiError::bad_request)?),
            None => return Err(ApiError::bad_request("dnat rule requires a target IP")),
        },
        other => {
            return Err(ApiError::bad_request(format!(
                "invalid kind `{other}` (allowed: masquerade | dnat)"
            )));
        }
    };

    let fw = state.service::<FirewallService>("firewall")?;
    let id = fw.add_nat_rule(
        &name,
        &kind,
        &interface,
        &source,
        &destination,
        &req.protocol,
        dest_port,
        target.as_deref(),
        &description,
    )?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id.to_string() })),
    )
        .into_response())
}

pub async fn delete_nat_rule(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let uuid = parse_uuid(&id, "rule")?;

    let fw = state.service::<FirewallService>("firewall")?;
    fw.remove_nat_rule(uuid)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Network: Zones ────────────────────────────────────

pub async fn list_zones(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<ZoneDto>>> {
    let net = state.service::<NetworkService>("network")?;

    let dtos = net
        .list_zones()?
        .into_iter()
        .map(|z| ZoneDto {
            id: z.id,
            name: z.name,
            display_name: z.display_name,
            accounting: z.accounting,
            description: z.description,
        })
        .collect();

    Ok(Json(dtos))
}

pub async fn create_zone(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateZoneRequest>,
) -> ApiResult<Response> {
    let net = state.service::<NetworkService>("network")?;

    // ── اعتبارسنجی ورودی‌ها
    let name = validation::check_zone_name(&req.name)
        .map_err(ApiError::bad_request)?
        .to_string();
    let display_name = validation::check_name(&req.display_name)
        .map_err(ApiError::bad_request)?
        .to_string();
    let description = validation::check_description(&req.description)
        .map_err(ApiError::bad_request)?
        .to_string();

    let id = net.create_zone(&name, &display_name, req.accounting, &description)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": id.to_string() })),
    )
        .into_response())
}

pub async fn update_zone(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<UpdateZoneRequest>,
) -> ApiResult<StatusCode> {
    let net = state.service::<NetworkService>("network")?;

    // ── اعتبارسنجی ورودی‌ها
    let display_name = validation::check_name(&req.display_name)
        .map_err(ApiError::bad_request)?
        .to_string();
    let description = validation::check_description(&req.description)
        .map_err(ApiError::bad_request)?
        .to_string();

    net.update_zone(&name, &display_name, req.accounting, &description)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_zone(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let net = state.service::<NetworkService>("network")?;
    net.delete_zone(&name)?;

    Ok(StatusCode::NO_CONTENT)
}

// ─── Network: Interfaces ───────────────────────────────

// ─── Network: Summary (N1) ─────────────────────────────

pub async fn network_summary(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<crate::dto::NetworkSummaryDto>> {
    let net = state.service::<NetworkService>("network")?;
    let s = net.summary()?;

    Ok(Json(crate::dto::NetworkSummaryDto {
        total_interfaces: s.total_interfaces,
        up_interfaces: s.up_interfaces,
        down_interfaces: s.down_interfaces,
        zones: s.zones,
        rx_bytes: s.rx_bytes,
        tx_bytes: s.tx_bytes,
        generated_at: s.generated_at,
    }))
}

pub async fn list_network_interfaces(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<NetworkInterfaceDto>>> {
    let net = state.service::<NetworkService>("network")?;

    let dtos = net
        .list_interfaces()?
        .into_iter()
        .map(|i| NetworkInterfaceDto {
            name: i.name,
            zone: i.zone,
            up: i.up,
            addresses: i.addresses,
            enabled: i.enabled,
            ipv4_mode: i.ipv4_mode,
            address_cidr: i.address_cidr,
            gateway: i.gateway,
            description: i.description,
        })
        .collect();

    Ok(Json(dtos))
}

pub async fn set_network_zone(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<SetNetworkZoneRequest>,
) -> ApiResult<StatusCode> {
    // ── اعتبارسنجی ورودی‌ها
    let name = validation::check_iface_name(&req.name)
        .map_err(ApiError::bad_request)?
        .to_string();
    let zone = validation::check_zone_name(&req.zone)
        .map_err(ApiError::bad_request)?
        .to_string();

    let net = state.service::<NetworkService>("network")?;
    net.set_zone(&name, &zone)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn apply_interface_config(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<InterfaceConfigRequest>,
) -> ApiResult<StatusCode> {
    // ── اعتبارسنجی ورودی‌ها
    let name = validation::check_iface_name(&req.name)
        .map_err(ApiError::bad_request)?
        .to_string();
    let zone = validation::check_zone_name(&req.zone)
        .map_err(ApiError::bad_request)?
        .to_string();
    let ipv4_mode = validation::check_ipv4_mode(&req.ipv4_mode)
        .map_err(ApiError::bad_request)?
        .to_string();
    validation::check_interface_config(
        &ipv4_mode,
        req.address_cidr.as_deref(),
        req.gateway.as_deref(),
    )
    .map_err(ApiError::bad_request)?;
    let description = validation::check_description(&req.description)
        .map_err(ApiError::bad_request)?
        .to_string();

    let net = state.service::<NetworkService>("network")?;
    net.apply_interface_config(
        &name,
        &zone,
        req.enabled,
        &ipv4_mode,
        req.address_cidr.as_deref(),
        req.gateway.as_deref(),
        &description,
    )?;

    Ok(StatusCode::NO_CONTENT)
}


// ─── Network: [N1b] Interface Rename ───────────────────────────────

pub async fn rename_interface(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<crate::dto::RenameInterfaceRequest>,
) -> ApiResult<StatusCode> {
    // ── اعتبارسنجی (IFNAMSIZ-1 = 15 بایت، کاراکترهای مجاز)
    let old = validation::check_iface_name(&req.name)
        .map_err(ApiError::bad_request)?
        .to_string();
    let new = validation::check_iface_name(&req.new_name)
        .map_err(ApiError::bad_request)?
        .to_string();

    let net = state.service::<NetworkService>("network")?;
    net.rename_interface(&old, &new)?;

    Ok(StatusCode::NO_CONTENT)
}


// ─── Network: [N2] PPPoE ───────────────────────────────

fn pppoe_session_dto(x: network::models::PppoeSession) -> crate::dto::PppoeSessionDto {
    crate::dto::PppoeSessionDto {
        iface: x.iface,
        profile: x.profile,
        local_ip: x.local_ip,
        remote_ip: x.remote_ip,
        up: x.up,
    }
}

pub async fn pppoe_state(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<crate::dto::PppoeStateDto>> {
    let net = state.service::<NetworkService>("network")?;
    let s = net.pppoe_state()?;

    Ok(Json(crate::dto::PppoeStateDto {
        profiles: s
            .profiles
            .into_iter()
            .map(|p| crate::dto::PppoeProfileDto {
                name: p.name,
                parent_iface: p.parent_iface,
                username: p.username,
                mtu: p.mtu,
                default_route: p.default_route,
                keepalive: p.keepalive,
                enabled: p.enabled,
                description: p.description,
                session: p.session.map(pppoe_session_dto),
            })
            .collect(),
        sessions: s.sessions.into_iter().map(pppoe_session_dto).collect(),
    }))
}

pub async fn pppoe_save(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<crate::dto::PppoeSaveRequest>,
) -> ApiResult<StatusCode> {
    // ── اعتبارسنجی سبک (کامل در سرویس)
    let name = req.name.trim().to_lowercase();
    if name.is_empty()
        || name.len() > 32
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ApiError::bad_request("profile name: 1-32 chars, a-z 0-9 -"));
    }
    let parent = validation::check_iface_name(&req.parent_iface)
        .map_err(ApiError::bad_request)?
        .to_string();
    let description = validation::check_description(&req.description)
        .map_err(ApiError::bad_request)?
        .to_string();

    let net = state.service::<NetworkService>("network")?;
    net.pppoe_save(
        &name,
        &parent,
        &req.username,
        &req.password,
        req.mtu,
        req.default_route,
        req.keepalive,
        req.enabled,
        &description,
    )?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn pppoe_start(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let net = state.service::<NetworkService>("network")?;
    net.pppoe_start(&name)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn pppoe_stop(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let net = state.service::<NetworkService>("network")?;
    net.pppoe_stop(&name)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn pppoe_delete(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let net = state.service::<NetworkService>("network")?;
    net.pppoe_delete(&name)?;
    Ok(StatusCode::NO_CONTENT)
}


// ─── Network: [N3] VPN WAN (PPTP / L2TP) ──────────────

fn vpn_connection_dto(x: network::models::VpnConnection) -> crate::dto::VpnConnectionDto {
    crate::dto::VpnConnectionDto {
        name: x.name,
        kind: x.kind,
        server: x.server,
        username: x.username,
        mtu: x.mtu,
        default_route: x.default_route,
        keepalive: x.keepalive,
        enabled: x.enabled,
        description: x.description,
        session: x.session.map(pppoe_session_dto),
    }
}

pub async fn vpn_state(
    _user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<Json<crate::dto::VpnWanStateDto>> {
    let net = state.service::<NetworkService>("network")?;
    let s = net.vpn_state()?;

    Ok(Json(crate::dto::VpnWanStateDto {
        connections: s.connections.into_iter().map(vpn_connection_dto).collect(),
        sessions: s.sessions.into_iter().map(pppoe_session_dto).collect(),
    }))
}

pub async fn vpn_save(
    _user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<crate::dto::VpnSaveRequest>,
) -> ApiResult<StatusCode> {
    // ── اعتبارسنجی سبک (کامل در سرویس)
    let name = req.name.trim().to_lowercase();
    if name.is_empty()
        || name.len() > 32
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ApiError::bad_request("connection name: 1-32 chars, a-z 0-9 -"));
    }
    let kind = req.kind.trim().to_lowercase();
    if kind != "pptp" && kind != "l2tp" {
        return Err(ApiError::bad_request("kind must be pptp or l2tp"));
    }
    let description = validation::check_description(&req.description)
        .map_err(ApiError::bad_request)?
        .to_string();

    let net = state.service::<NetworkService>("network")?;
    net.vpn_save(
        &name,
        &kind,
        &req.server,
        &req.username,
        &req.password,
        req.mtu,
        req.default_route,
        req.keepalive,
        req.enabled,
        &description,
    )?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn vpn_start(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let net = state.service::<NetworkService>("network")?;
    net.vpn_start(&name)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn vpn_stop(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let net = state.service::<NetworkService>("network")?;
    net.vpn_stop(&name)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn vpn_delete(
    _user: AuthUser,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    let net = state.service::<NetworkService>("network")?;
    net.vpn_delete(&name)?;
    Ok(StatusCode::NO_CONTENT)
}
