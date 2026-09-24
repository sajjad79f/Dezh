use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StaticRouteRow {
    pub id: Uuid,
    pub destination: String,
    pub gateway: Option<String>,
    pub device: Option<String>,
    pub description: String,
    pub enabled: bool,
}

pub struct StaticRouteRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> StaticRouteRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, r: &StaticRouteRow) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO static_routes (id, destination, gateway, device, description, enabled)
            VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(r.id)
        .bind(&r.destination)
        .bind(&r.gateway)
        .bind(&r.device)
        .bind(&r.description)
        .bind(r.enabled)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> StorageResult<Vec<StaticRouteRow>> {
        let rows = sqlx::query_as::<_, StaticRouteRow>(
            r#"
            SELECT id, destination, gateway, device, description, enabled
            FROM static_routes
            ORDER BY destination
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete(&self, id: Uuid) -> StorageResult<()> {
        let result = sqlx::query(r#"DELETE FROM static_routes WHERE id = $1"#)
            .bind(id)
            .execute(self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }
        Ok(())
    }
}