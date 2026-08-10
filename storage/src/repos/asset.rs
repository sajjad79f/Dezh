use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AssetRow {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub status: String,
}

pub struct AssetRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> AssetRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        id: Uuid,
        name: &str,
        asset_type: &str,
        status: &str,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO assets (id, name, asset_type, status)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(asset_type)
        .bind(status)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> StorageResult<Vec<AssetRow>> {
        let rows = sqlx::query_as::<_, AssetRow>(
            r#"SELECT id, name, asset_type, status FROM assets ORDER BY name"#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_name(&self, name: &str) -> StorageResult<Option<AssetRow>> {
        let row = sqlx::query_as::<_, AssetRow>(
            r#"SELECT id, name, asset_type, status FROM assets WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn set_status(&self, id: Uuid, status: &str) -> StorageResult<()> {
        let result = sqlx::query(
            r#"UPDATE assets SET status = $2, updated_at = NOW() WHERE id = $1"#,
        )
        .bind(id)
        .bind(status)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }
        Ok(())
    }
}