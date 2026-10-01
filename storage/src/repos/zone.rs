use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ZoneRow {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub accounting: bool,
    pub description: String,
}

pub struct ZoneRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> ZoneRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> StorageResult<Vec<ZoneRow>> {
        Ok(sqlx::query_as::<_, ZoneRow>(
            r#"SELECT id, name, display_name, accounting, description FROM zones ORDER BY name"#,
        )
        .fetch_all(self.pool)
        .await?)
    }

    pub async fn find_by_name(&self, name: &str) -> StorageResult<Option<ZoneRow>> {
        Ok(sqlx::query_as::<_, ZoneRow>(
            r#"SELECT id, name, display_name, accounting, description FROM zones WHERE name = $1"#,
        )
        .bind(name)
        .fetch_optional(self.pool)
        .await?)
    }

    pub async fn create(
        &self,
        name: &str,
        display_name: &str,
        accounting: bool,
        description: &str,
    ) -> StorageResult<Uuid> {
        let name = name.trim().to_lowercase();
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(StorageError::Message(
                "zone name: only a-z 0-9 _ -".into(),
            ));
        }
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO zones (id, name, display_name, accounting, description)
            VALUES ($1,$2,$3,$4,$5)
            "#,
        )
        .bind(id)
        .bind(&name)
        .bind(display_name)
        .bind(accounting)
        .bind(description)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        name: &str,
        display_name: &str,
        accounting: bool,
        description: &str,
    ) -> StorageResult<()> {
        let r = sqlx::query(
            r#"
            UPDATE zones
            SET display_name = $2, accounting = $3, description = $4
            WHERE name = $1
            "#,
        )
        .bind(name)
        .bind(display_name)
        .bind(accounting)
        .bind(description)
        .execute(self.pool)
        .await?;
        if r.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.into()));
        }
        Ok(())
    }

    pub async fn delete(&self, name: &str) -> StorageResult<()> {
        let r = sqlx::query(r#"DELETE FROM zones WHERE name = $1"#)
            .bind(name)
            .execute(self.pool)
            .await?;
        if r.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.into()));
        }
        Ok(())
    }

    pub async fn accounting_zone_names(&self) -> StorageResult<Vec<String>> {
        let rows: Vec<(String,)> =
            sqlx::query_as(r#"SELECT name FROM zones WHERE accounting = TRUE"#)
                .fetch_all(self.pool)
                .await?;
        Ok(rows.into_iter().map(|(n,)| n).collect())
    }
}