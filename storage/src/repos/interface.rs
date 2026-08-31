use sqlx::PgPool;

use crate::error::StorageResult;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InterfaceRow {
    pub name: String,
    pub zone: String,
}

pub struct InterfaceRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> InterfaceRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> StorageResult<Vec<InterfaceRow>> {
        let rows = sqlx::query_as::<_, InterfaceRow>(
            r#"SELECT name, zone FROM network_interfaces ORDER BY name"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn ensure_known(&self, name: &str) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO network_interfaces (name, zone)
            VALUES ($1, 'lan')
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .bind(name)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_zone(&self, name: &str, zone: &str) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO network_interfaces (name, zone, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (name) DO UPDATE
              SET zone = EXCLUDED.zone, updated_at = NOW()
            "#,
        )
        .bind(name)
        .bind(zone)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}