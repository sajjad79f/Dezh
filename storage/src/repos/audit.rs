use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageResult;

/// سطر audit برای نمایش در صفحه‌ی Security Events
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditEventRow {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub actor: Option<String>,
    pub action: String,
    pub detail: serde_json::Value,
}

pub struct AuditRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> AuditRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn log(
        &self,
        actor: Option<&str>,
        action: &str,
        resource: Option<&str>,
        detail: serde_json::Value,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_log (id, actor, action, resource, detail)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(actor)
        .bind(action)
        .bind(resource)
        .bind(detail)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    /// رخدادهای امنیتیِ audit (ورود ناموفق / مسدودسازی IP / تغییر قوانین)
    /// برای ادغام با security_events در صفحه‌ی Events
    pub async fn list_recent(
        &self,
        since_hours: i64,
        limit: i64,
    ) -> StorageResult<Vec<AuditEventRow>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(since_hours.clamp(1, 24 * 30));
        let actions: &[&str] = &[
            "login",
            "login_failed",
            "login_rate_limited",
            "firewall_rule_created",
            "firewall_rule_deleted",
            "firewall_rule_updated",
        ];

        let rows = sqlx::query_as::<_, AuditEventRow>(
            r#"
            SELECT created_at, actor, action, detail
            FROM audit_log
            WHERE created_at > $1 AND action = ANY($2)
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(since)
        .bind(actions)
        .bind(limit.clamp(1, 1000))
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }
}