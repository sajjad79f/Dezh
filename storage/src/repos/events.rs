use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageResult;

/// یک رخداد امنیتی ذخیره‌شده (hit قانون / تلاش ورود ناموفق / ...)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SecurityEventRow {
    pub id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub kind: String,
    pub severity: String,
    pub source_ip: Option<String>,
    pub dest_port: Option<i32>,
    pub rule_id: Option<Uuid>,
    pub hits: i64,
    pub message: String,
}

/// رخداد جدید برای insert (از کالکتور kmsg یا آینده از منابع دیگر)
#[derive(Debug, Clone)]
pub struct NewSecurityEvent {
    pub kind: String,
    pub severity: String,
    pub source_ip: Option<String>,
    pub dest_port: Option<u16>,
    pub rule_id: Option<Uuid>,
    pub hits: u64,
    pub message: String,
}

pub struct EventsRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> EventsRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// درج دسته‌ای داخل یک تراکنش — کالکتور هر ۵ ثانیه یک بار صدا می‌زند
    pub async fn insert_many(&self, events: &[NewSecurityEvent]) -> StorageResult<u64> {
        if events.is_empty() {
            return Ok(0);
        }

        let mut tx = self.pool.begin().await?;
        let mut n = 0u64;

        for e in events {
            sqlx::query(
                r#"
                INSERT INTO security_events
                    (kind, severity, source_ip, dest_port, rule_id, hits, message)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(&e.kind)
            .bind(&e.severity)
            .bind(&e.source_ip)
            .bind(e.dest_port.map(|p| p as i32))
            .bind(e.rule_id)
            .bind(e.hits as i64)
            .bind(&e.message)
            .execute(&mut *tx)
            .await?;
            n += 1;
        }

        tx.commit().await?;
        Ok(n)
    }

    /// آخرین رخدادها (با فیلتر اختیاری kind)
    pub async fn list(
        &self,
        limit: i64,
        kind: Option<&str>,
        since_hours: i64,
    ) -> StorageResult<Vec<SecurityEventRow>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(since_hours.clamp(1, 24 * 30));
        let lim = limit.clamp(1, 1000);

        let rows = match kind {
            Some(k) => {
                sqlx::query_as::<_, SecurityEventRow>(
                    r#"
                    SELECT id, created_at, kind, severity, source_ip, dest_port, rule_id, hits, message
                    FROM security_events
                    WHERE created_at > $1 AND kind = $2
                    ORDER BY created_at DESC
                    LIMIT $3
                    "#,
                )
                .bind(since)
                .bind(k)
                .bind(lim)
                .fetch_all(self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, SecurityEventRow>(
                    r#"
                    SELECT id, created_at, kind, severity, source_ip, dest_port, rule_id, hits, message
                    FROM security_events
                    WHERE created_at > $1
                    ORDER BY created_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(since)
                .bind(lim)
                .fetch_all(self.pool)
                .await?
            }
        };

        Ok(rows)
    }

    /// تعداد رخداد به تفکیک kind
    pub async fn count_by_kind(&self, since_hours: i64) -> StorageResult<Vec<(String, i64)>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(since_hours.clamp(1, 24 * 30));
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT kind, count(*) AS c
            FROM security_events
            WHERE created_at > $1
            GROUP BY kind
            ORDER BY c DESC
            "#,
        )
        .bind(since)
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    /// منابع مهاجم (بیشترین hit)
    pub async fn top_sources(&self, since_hours: i64, limit: i64) -> StorageResult<Vec<(String, i64)>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(since_hours.clamp(1, 24 * 30));
        let rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            SELECT source_ip, sum(hits) AS total
            FROM security_events
            WHERE created_at > $1 AND source_ip IS NOT NULL
            GROUP BY source_ip
            ORDER BY total DESC
            LIMIT $2
            "#,
        )
        .bind(since)
        .bind(limit.clamp(1, 100))
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    /// پورت‌های هدف (بیشترین hit)
    pub async fn top_ports(&self, since_hours: i64, limit: i64) -> StorageResult<Vec<(i32, i64)>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(since_hours.clamp(1, 24 * 30));
        let rows = sqlx::query_as::<_, (i32, i64)>(
            r#"
            SELECT dest_port, sum(hits) AS total
            FROM security_events
            WHERE created_at > $1 AND dest_port IS NOT NULL
            GROUP BY dest_port
            ORDER BY total DESC
            LIMIT $2
            "#,
        )
        .bind(since)
        .bind(limit.clamp(1, 100))
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    /// تایم‌لاین ساعتی برای نمودار صفحه‌ی Events
    pub async fn timeline_hourly(
        &self,
        since_hours: i64,
    ) -> StorageResult<Vec<(chrono::DateTime<chrono::Utc>, i64)>> {
        let since = chrono::Utc::now() - chrono::Duration::hours(since_hours.clamp(1, 24 * 30));
        let rows = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, i64)>(
            r#"
            SELECT date_trunc('hour', created_at) AS bucket, sum(hits) AS total
            FROM security_events
            WHERE created_at > $1
            GROUP BY bucket
            ORDER BY bucket
            "#,
        )
        .bind(since)
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    /// پاکسازی رخدادهای قدیمی — کالکتور هر ساعت صدا می‌زند
    pub async fn prune_older_than_days(&self, days: i64) -> StorageResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM security_events
            WHERE created_at < now() - make_interval(days => $1)
            "#,
        )
        .bind(days.clamp(1, 365) as i32)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}