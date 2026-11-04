use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FirewallRuleRow {
    pub id: Uuid,
    pub name: String,
    pub action: String,
    pub direction: String,
    pub protocol: String,
    pub source: String,
    pub destination: String,
    pub port: Option<i32>,
    pub enabled: bool,
    pub priority: i32,
    pub description: String,
    pub log: bool,
    pub source_set: Option<String>,
    pub dest_set: Option<String>,
    /// سقف تطبیق قانون در ثانیه (limit rate) — NULL = بی‌سقف
    pub rate_limit: Option<i32>,
    /// فاز ۴: زمان‌بندی اجرا
    pub schedule_enabled: bool,
    pub schedule_start: String,
    pub schedule_end: String,
    pub schedule_days: String,
    /// فاز ۵: ارجاع به شیء دامنه (با «نام» مثل source_set) — NULL = بدون FQDN
    pub source_fqdn: Option<String>,
    pub dest_fqdn: Option<String>,
}

pub struct FirewallRepo<'a> {
    pool: &'a PgPool,
}

const RULE_COLUMNS: &str = "id, name, action, direction, protocol, source, destination, port, \
                            enabled, priority, description, log, source_set, dest_set, rate_limit, \
                            schedule_enabled, schedule_start, schedule_end, schedule_days, \
                            source_fqdn, dest_fqdn";

impl<'a> FirewallRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, rule: &FirewallRuleRow) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO firewall_rules
                (id, name, action, direction, protocol, source, destination, port,
                 enabled, priority, description, log, source_set, dest_set, rate_limit,
                 schedule_enabled, schedule_start, schedule_end, schedule_days,
                 source_fqdn, dest_fqdn)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
            "#,
        )
        .bind(rule.id)
        .bind(&rule.name)
        .bind(&rule.action)
        .bind(&rule.direction)
        .bind(&rule.protocol)
        .bind(&rule.source)
        .bind(&rule.destination)
        .bind(rule.port)
        .bind(rule.enabled)
        .bind(rule.priority)
        .bind(&rule.description)
        .bind(rule.log)
        .bind(&rule.source_set)
        .bind(&rule.dest_set)
        .bind(rule.rate_limit)
        .bind(rule.schedule_enabled)
        .bind(&rule.schedule_start)
        .bind(&rule.schedule_end)
        .bind(&rule.schedule_days)
        .bind(&rule.source_fqdn)
        .bind(&rule.dest_fqdn)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> StorageResult<Vec<FirewallRuleRow>> {
        let rows = sqlx::query_as::<_, FirewallRuleRow>(&format!(
            r#"
            SELECT {RULE_COLUMNS}
            FROM firewall_rules
            ORDER BY priority, name
            "#
        ))
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    /// به‌روزرسانی کامل یک قانون (PATCH)
    pub async fn update(&self, rule: &FirewallRuleRow) -> StorageResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE firewall_rules
            SET name = $2, action = $3, direction = $4, protocol = $5,
                source = $6, destination = $7, port = $8, enabled = $9,
                priority = $10, description = $11, log = $12,
                source_set = $13, dest_set = $14, rate_limit = $15,
                schedule_enabled = $16, schedule_start = $17, schedule_end = $18, schedule_days = $19,
                source_fqdn = $20, dest_fqdn = $21
            WHERE id = $1
            "#,
        )
        .bind(rule.id)
        .bind(&rule.name)
        .bind(&rule.action)
        .bind(&rule.direction)
        .bind(&rule.protocol)
        .bind(&rule.source)
        .bind(&rule.destination)
        .bind(rule.port)
        .bind(rule.enabled)
        .bind(rule.priority)
        .bind(&rule.description)
        .bind(rule.log)
        .bind(&rule.source_set)
        .bind(&rule.dest_set)
        .bind(rule.rate_limit)
        .bind(rule.schedule_enabled)
        .bind(&rule.schedule_start)
        .bind(&rule.schedule_end)
        .bind(&rule.schedule_days)
        .bind(&rule.source_fqdn)
        .bind(&rule.dest_fqdn)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(rule.id.to_string()));
        }
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> StorageResult<()> {
        let result = sqlx::query(r#"DELETE FROM firewall_rules WHERE id = $1"#)
            .bind(id)
            .execute(self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }
        Ok(())
    }
}

// ─── IP Sets (Address Lists) ───────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IpSetRow {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub description: String,
    pub enabled: bool,
    pub entries: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct IpSetRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> IpSetRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, set: &IpSetRow) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO ip_sets (id, name, kind, description, enabled, entries)
            VALUES ($1,$2,$3,$4,$5,$6)
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .bind(set.id)
        .bind(&set.name)
        .bind(&set.kind)
        .bind(&set.description)
        .bind(set.enabled)
        .bind(&set.entries)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> StorageResult<Vec<IpSetRow>> {
        let rows = sqlx::query_as::<_, IpSetRow>(
            r#"
            SELECT id, name, kind, description, enabled, entries, created_at
            FROM ip_sets
            ORDER BY kind, name
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_name(&self, name: &str) -> StorageResult<Option<IpSetRow>> {
        let row = sqlx::query_as::<_, IpSetRow>(
            r#"
            SELECT id, name, kind, description, enabled, entries, created_at
            FROM ip_sets
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_entries(&self, name: &str, entries: &[String]) -> StorageResult<()> {
        let result = sqlx::query(r#"UPDATE ip_sets SET entries = $2 WHERE name = $1"#)
            .bind(name)
            .bind(entries)
            .execute(self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.to_string()));
        }
        Ok(())
    }

    pub async fn set_enabled(&self, name: &str, enabled: bool) -> StorageResult<()> {
        let result = sqlx::query(r#"UPDATE ip_sets SET enabled = $2 WHERE name = $1"#)
            .bind(name)
            .bind(enabled)
            .execute(self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.to_string()));
        }
        Ok(())
    }

    pub async fn delete(&self, name: &str) -> StorageResult<()> {
        let result = sqlx::query(r#"DELETE FROM ip_sets WHERE name = $1"#)
            .bind(name)
            .execute(self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.to_string()));
        }
        Ok(())
    }
}

// ─── NAT ───────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NatRuleRow {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub kind: String,
    pub interface: String,
    pub source: String,
    pub destination: String,
    pub protocol: String,
    pub dest_port: Option<i32>,
    pub target: Option<String>,
    pub description: String,
}

pub struct NatRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> NatRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, r: &NatRuleRow) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO nat_rules
              (id, name, enabled, kind, interface, source, destination, protocol, dest_port, target, description)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            "#,
        )
        .bind(r.id)
        .bind(&r.name)
        .bind(r.enabled)
        .bind(&r.kind)
        .bind(&r.interface)
        .bind(&r.source)
        .bind(&r.destination)
        .bind(&r.protocol)
        .bind(r.dest_port)
        .bind(&r.target)
        .bind(&r.description)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> StorageResult<Vec<NatRuleRow>> {
        let rows = sqlx::query_as::<_, NatRuleRow>(
            r#"
            SELECT id, name, enabled, kind, interface, source, destination,
                   protocol, dest_port, target, description
            FROM nat_rules
            ORDER BY kind, name
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete(&self, id: Uuid) -> StorageResult<()> {
        let result = sqlx::query(r#"DELETE FROM nat_rules WHERE id = $1"#)
            .bind(id)
            .execute(self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }
        Ok(())
    }
}