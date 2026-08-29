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

    /// Creates the row on first sight of an interface (default zone
    /// "lan"), or does nothing if it's already known -- never
    /// overwrites a zone an admin already assigned.
    pub async fn ensure_known(&self, name: &str) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO interfaces (name, zone)
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
            INSERT INTO interfaces (name, zone)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET zone = $2, updated_at = now()
            "#,
        )
        .bind(name)
        .bind(zone)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn list(&self) -> StorageResult<Vec<InterfaceRow>> {
        let rows = sqlx::query_as::<_, InterfaceRow>(
            r#"SELECT name, zone FROM interfaces ORDER BY name"#,
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn get_zone(&self, name: &str) -> StorageResult<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as(r#"SELECT zone FROM interfaces WHERE name = $1"#)
                .bind(name)
                .fetch_optional(self.pool)
                .await?;

        Ok(row.map(|(z,)| z))
    }
}