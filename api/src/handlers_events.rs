//! فاز ۳ — Security Events endpoints
//!
//! منابع رخداد:
//! ۱. `security_events` — hit قوانین (از کالکتور kmsg)
//! ۲. `audit_log` — تلاش‌های ورود ناموفق / مسدودسازی IP / تغییر قوانین
//!
//! خروجی دو منبع در یک استریم یکپارچه merge می‌شود (مرتب بر اساس زمان).

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::auth::AuthUser;
use crate::dto::{
    EventKindCount, EventPortCount, EventSourceCount, EventSummaryDto, EventTimePoint,
    SecurityEventDto,
};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use firewall::FirewallService;
use storage::{AuditRepo, DbPool, EventsRepo};

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    /// حداکثر تعداد (پیش‌فرض ۲۰۰، سقف ۱۰۰۰)
    pub limit: Option<i64>,
    /// بازه به ساعت (پیش‌فرض ۲۴)
    pub hours: Option<i64>,
    /// فیلتر kind اختیاری: rule_hit | login_failed | login | ...
    pub kind: Option<String>,
}

fn severity_of_action(action: &str) -> &'static str {
    match action {
        "login_rate_limited" => "critical",
        "login_failed" => "warning",
        _ => "info",
    }
}

fn is_login_kind(kind: &str) -> bool {
    matches!(
        kind,
        "login" | "login_failed" | "login_rate_limited"
    )
}

/// GET /api/events — آخرین رخدادهای امنیتی (ادغام kernel hits + audit)
pub async fn list_events(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventsQuery>,
) -> ApiResult<Json<Vec<SecurityEventDto>>> {
    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let hours = q.hours.unwrap_or(24);
    let limit = q.limit.unwrap_or(200);
    let kind = q.kind.as_deref();

    let rows = EventsRepo::new(pool.inner())
        .list(limit, kind.filter(|k| !is_login_kind(k)), hours)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // نام قوانین از registry — rename شدن قانون هم روی رخدادهای قبلی اعمال می‌شود
    let rule_names: HashMap<uuid::Uuid, String> = state
        .services
        .resolve::<FirewallService>()
        .map(|fw| {
            fw.list_rules()
                .into_iter()
                .map(|r| (r.id, r.name))
                .collect()
        })
        .unwrap_or_default();

    let mut merged: Vec<SecurityEventDto> = rows
        .iter()
        .map(|e| SecurityEventDto {
            id: e.id.to_string(),
            created_at: e.created_at.to_rfc3339(),
            kind: e.kind.clone(),
            severity: e.severity.clone(),
            source_ip: e.source_ip.clone(),
            dest_port: e.dest_port.and_then(|p| u16::try_from(p).ok()),
            rule_name: e.rule_id.and_then(|id| rule_names.get(&id).cloned()),
            hits: e.hits.max(0) as u64,
            message: e.message.clone(),
        })
        .collect();

    // رخدادهای ورود از audit — مگر وقتی فیلتر kind چیز دیگری خواسته باشد
    if kind.is_none() || kind.is_some_and(is_login_kind) {
        if let Ok(audit_rows) = AuditRepo::new(pool.inner()).list_recent(hours, limit).await {
            for r in audit_rows {
                if let Some(k) = kind {
                    if r.action != k {
                        continue;
                    }
                }

                let ip = r
                    .detail
                    .get("ip")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let message = match r.action.as_str() {
                    "login_failed" => format!(
                        "failed login for '{}' ({})",
                        r.actor.as_deref().unwrap_or("unknown"),
                        r.detail
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("invalid credentials"),
                    ),
                    "login_rate_limited" => format!(
                        "IP {} temporarily blocked by brute-force protection",
                        ip.as_deref().unwrap_or("?"),
                    ),
                    "login" => format!(
                        "'{}' signed in",
                        r.actor.as_deref().unwrap_or("?"),
                    ),
                    "firewall_rule_created" => format!(
                        "rule '{}' created",
                        r.detail.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
                    ),
                    "firewall_rule_deleted" => "a firewall rule was deleted".to_string(),
                    "firewall_rule_updated" => "a firewall rule was updated".to_string(),
                    other => other.to_string(),
                };

                merged.push(SecurityEventDto {
                    id: format!("audit-{}", r.created_at.to_rfc3339()),
                    created_at: r.created_at.to_rfc3339(),
                    kind: r.action.clone(),
                    severity: severity_of_action(&r.action).into(),
                    source_ip: ip,
                    dest_port: None,
                    rule_name: None,
                    hits: 1,
                    message,
                });
            }
        }
    }

    // فیلتر kind — در Rust اعمال می‌شود تا هم رخدادهای DB و هم audit را بپوشاند
    if let Some(k) = kind {
        merged.retain(|e| e.kind == k);
    }

    merged.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    merged.truncate(limit.clamp(1, 1000) as usize);

    Ok(Json(merged))
}

/// GET /api/events/summary — خلاصه برای نمودارهای صفحه‌ی Events
pub async fn events_summary(
    _user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<EventsQuery>,
) -> ApiResult<Json<EventSummaryDto>> {
    let pool = state
        .services
        .resolve::<DbPool>()
        .ok_or_else(|| ApiError::service_unavailable("database unavailable"))?;

    let hours = q.hours.unwrap_or(24).clamp(1, 24 * 30);
    let repo = EventsRepo::new(pool.inner());

    let by_kind = repo
        .count_by_kind(hours)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let top_sources = repo
        .top_sources(hours, 10)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let top_ports = repo
        .top_ports(hours, 10)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let timeline = repo
        .timeline_hourly(hours)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let total_hits: u64 = timeline.iter().map(|(_, c)| (*c).max(0) as u64).sum();

    // ── ادغام تلاش‌های ناموفق ورود (audit) در by_kind و top_sources
    let mut kinds: Vec<EventKindCount> = by_kind
        .into_iter()
        .map(|(k, c)| EventKindCount {
            kind: k,
            count: c.max(0) as u64,
        })
        .collect();

    let mut sources: HashMap<String, u64> = top_sources
        .into_iter()
        .map(|(ip, c)| (ip, c.max(0) as u64))
        .collect();

    let mut login_failed = 0u64;
    if let Ok(audit_rows) = AuditRepo::new(pool.inner()).list_recent(hours, 1000).await {
        for r in audit_rows.iter().filter(|r| r.action == "login_failed") {
            login_failed += 1;
            if let Some(ip) = r.detail.get("ip").and_then(|v| v.as_str()) {
                *sources.entry(ip.to_string()).or_insert(0) += 1;
            }
        }
    }
    if login_failed > 0 {
        kinds.push(EventKindCount {
            kind: "login_failed".into(),
            count: login_failed,
        });
    }
    kinds.sort_by(|a, b| b.count.cmp(&a.count));

    let mut top: Vec<EventSourceCount> = sources
        .into_iter()
        .map(|(ip, count)| EventSourceCount { ip, count })
        .collect();
    top.sort_by(|a, b| b.count.cmp(&a.count));
    top.truncate(10);

    Ok(Json(EventSummaryDto {
        since_hours: hours as u32,
        total_hits,
        by_kind: kinds,
        top_sources: top,
        top_ports: top_ports
            .into_iter()
            .filter_map(|(p, c)| {
                u16::try_from(p)
                    .ok()
                    .map(|port| EventPortCount {
                        port,
                        count: c.max(0) as u64,
                    })
            })
            .collect(),
        timeline: timeline
            .into_iter()
            .map(|(h, c)| EventTimePoint {
                hour: h.to_rfc3339(),
                count: c.max(0) as u64,
            })
            .collect(),
    }))
}