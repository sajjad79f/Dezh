use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageResult;

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
}