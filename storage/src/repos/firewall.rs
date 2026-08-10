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
}

pub struct FirewallRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> FirewallRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, rule: &FirewallRuleRow) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO firewall_rules
                (id, name, action, direction, protocol, source, destination, port, enabled, priority)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
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
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> StorageResult<Vec<FirewallRuleRow>> {
        let rows = sqlx::query_as::<_, FirewallRuleRow>(
            r#"
            SELECT id, name, action, direction, protocol, source, destination, port, enabled, priority
            FROM firewall_rules
            ORDER BY priority, name
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
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