use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageResult;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IdentityRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub source: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountingSessionRow {
    pub id: Uuid,
    pub identity_id: Option<Uuid>,
    pub protocol: String,
    pub ip_address: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub bytes_in: i64,
    pub bytes_out: i64,
    pub metadata: serde_json::Value,
}

pub struct IdentityRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> IdentityRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        username: &str,
        display_name: Option<&str>,
        source: &str,
    ) -> StorageResult<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO identities (id, username, display_name, source)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(display_name)
        .bind(source)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    pub async fn list(&self) -> StorageResult<Vec<IdentityRow>> {
        let rows = sqlx::query_as::<_, IdentityRow>(
            r#"
            SELECT id, username, display_name, source, enabled
            FROM identities ORDER BY username
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_username(&self, username: &str) -> StorageResult<Option<IdentityRow>> {
        let row = sqlx::query_as::<_, IdentityRow>(
            r#"
            SELECT id, username, display_name, source, enabled
            FROM identities WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn set_enabled(&self, id: Uuid, enabled: bool) -> StorageResult<()> {
        sqlx::query("UPDATE identities SET enabled = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(enabled)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

pub struct AccountingRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> AccountingRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn start_session(
        &self,
        identity_id: Option<Uuid>,
        protocol: &str,
        ip_address: Option<&str>,
    ) -> StorageResult<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO accounting_sessions (id, identity_id, protocol, ip_address)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(identity_id)
        .bind(protocol)
        .bind(ip_address)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    pub async fn end_session(
        &self,
        session_id: Uuid,
        bytes_in: i64,
        bytes_out: i64,
        cause: Option<&str>,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            UPDATE accounting_sessions
            SET ended_at = NOW(),
                bytes_in = $2,
                bytes_out = $3,
                terminate_cause = $4
            WHERE id = $1 AND ended_at IS NULL
            "#,
        )
        .bind(session_id)
        .bind(bytes_in)
        .bind(bytes_out)
        .bind(cause)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    /// مثل start_session، اما metadata (hostname/os) هم ذخیره می‌کند
    pub async fn start_session_meta(
        &self,
        identity_id: Option<Uuid>,
        protocol: &str,
        ip_address: Option<&str>,
        metadata: serde_json::Value,
    ) -> StorageResult<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO accounting_sessions (id, identity_id, protocol, ip_address, metadata)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(identity_id)
        .bind(protocol)
        .bind(ip_address)
        .bind(metadata)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    /// نشست بازِ یک identity به همراه IP آن — برای منطق reuse هوشمند
    pub async fn find_open_session_with_ip(
        &self,
        identity_id: Uuid,
        protocol: &str,
    ) -> StorageResult<Option<(Uuid, Option<String>)>> {
        let row: Option<(Uuid, Option<String>)> = sqlx::query_as(
            r#"
            SELECT id, ip_address FROM accounting_sessions
            WHERE identity_id = $1 AND protocol = $2 AND ended_at IS NULL
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(identity_id)
        .bind(protocol)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    /// شناسه همه نشست‌های باز — برای interim loop
    pub async fn list_open_ids(&self) -> StorageResult<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"SELECT id FROM accounting_sessions WHERE ended_at IS NULL"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// نوشتن مقادیر مطلق counterهای nft در DB (دوره‌ای)
    pub async fn update_interim(
        &self,
        session_id: Uuid,
        bytes_in: i64,
        bytes_out: i64,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            UPDATE accounting_sessions
            SET bytes_in = $2, bytes_out = $3, last_updated = NOW()
            WHERE id = $1 AND ended_at IS NULL
            "#,
        )
        .bind(session_id)
        .bind(bytes_in)
        .bind(bytes_out)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    /// آخرین بایت‌های ثبت‌شده (fallback وقتی counter از دست رفته)
    pub async fn get_bytes(&self, session_id: Uuid) -> StorageResult<(i64, i64)> {
        let row: Option<(i64, i64)> = sqlx::query_as(
            r#"SELECT bytes_in, bytes_out FROM accounting_sessions WHERE id = $1"#,
        )
        .bind(session_id)
        .fetch_optional(self.pool)
        .await?;
        Ok(row.unwrap_or((0, 0)))
    }

    /// بستن نشست‌هایی که مدتی update نداشته‌اند
    pub async fn close_stale(&self, idle_secs: i32, cause: &str) -> StorageResult<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            UPDATE accounting_sessions
            SET ended_at = NOW(), terminate_cause = $2
            WHERE ended_at IS NULL
              AND last_updated < NOW() - make_interval(secs => $1::int)
            RETURNING id
            "#,
        )
        .bind(idle_secs)
        .bind(cause)
        .fetch_all(self.pool)
        .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// تاریخچه نشست‌های بسته‌شده با فیلترهای اختیاری
    pub async fn list_history(
        &self,
        identity_id: Option<Uuid>,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
        offset: i64,
    ) -> StorageResult<Vec<AccountingSessionRow>> {
        let mut qb = sqlx::query_builder::QueryBuilder::new(
            r#"
            SELECT id, identity_id, protocol, ip_address, started_at, ended_at,
                   bytes_in, bytes_out, metadata
            FROM accounting_sessions
            WHERE ended_at IS NOT NULL
            "#,
        );

        if let Some(id) = identity_id {
            qb.push(" AND identity_id = ");
            qb.push_bind(id);
        }
        if let Some(f) = from {
            qb.push(" AND started_at >= ");
            qb.push_bind(f);
        }
        if let Some(t) = to {
            qb.push(" AND started_at <= ");
            qb.push_bind(t);
        }

        qb.push(" ORDER BY started_at DESC LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let rows = qb
            .build_query_as::<AccountingSessionRow>()
            .fetch_all(self.pool)
            .await?;
        Ok(rows)
    }

    /// خلاصه مصرف به تفکیک identity
    pub async fn usage_summary(
        &self,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> StorageResult<Vec<UsageRow>> {
        let mut qb = sqlx::query_builder::QueryBuilder::new(
            r#"
            SELECT s.identity_id,
                   i.username,
                   COUNT(*) AS sessions,
                   COALESCE(SUM(s.bytes_in), 0) AS bytes_in,
                   COALESCE(SUM(s.bytes_out), 0) AS bytes_out
            FROM accounting_sessions s
            LEFT JOIN identities i ON i.id = s.identity_id
            WHERE s.ended_at IS NOT NULL
            "#,
        );

        if let Some(f) = from {
            qb.push(" AND s.started_at >= ");
            qb.push_bind(f);
        }
        if let Some(t) = to {
            qb.push(" AND s.started_at <= ");
            qb.push_bind(t);
        }

        qb.push(" GROUP BY s.identity_id, i.username ORDER BY bytes_in + bytes_out DESC");

        let rows = qb.build_query_as::<UsageRow>().fetch_all(self.pool).await?;
        Ok(rows)
    }

    pub async fn end_open_sessions_for_identity(
        &self,
        identity_id: Uuid,
        bytes_in: i64,
        bytes_out: i64,
        cause: Option<&str>,
    ) -> StorageResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE accounting_sessions
            SET ended_at = NOW(),
                bytes_in = $2,
                bytes_out = $3,
                terminate_cause = $4
            WHERE identity_id = $1 AND ended_at IS NULL
            "#,
        )
        .bind(identity_id)
        .bind(bytes_in)
        .bind(bytes_out)
        .bind(cause)
        .execute(self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_open_session(
        &self,
        identity_id: Uuid,
        protocol: &str,
    ) -> StorageResult<Option<Uuid>> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM accounting_sessions
            WHERE identity_id = $1 AND protocol = $2 AND ended_at IS NULL
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(identity_id)
        .bind(protocol)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|(id,)| id))
    }

    pub async fn list_active(&self) -> StorageResult<Vec<AccountingSessionRow>> {
        let rows = sqlx::query_as::<_, AccountingSessionRow>(
            r#"
            SELECT id, identity_id, protocol, ip_address, started_at, ended_at,
                   bytes_in, bytes_out, metadata
            FROM accounting_sessions
            WHERE ended_at IS NULL
            ORDER BY started_at DESC
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }
    
    pub async fn list_open_for_identity(
        &self,
        identity_id: Uuid,
    ) -> StorageResult<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM accounting_sessions
            WHERE identity_id = $1 AND ended_at IS NULL
            "#,
        )
        .bind(identity_id)
        .fetch_all(self.pool)
        .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UsageRow {
    pub identity_id: Option<Uuid>,
    pub username: Option<String>,
    pub sessions: i64,
    pub bytes_in: i64,
    pub bytes_out: i64,
}