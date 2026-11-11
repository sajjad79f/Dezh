use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PppoeRow {
    pub id: Uuid,
    pub name: String,
    pub parent_iface: String,
    pub username: String,
    pub password: String,
    pub mtu: i32,
    pub default_route: bool,
    pub keepalive: i32,
    pub enabled: bool,
    pub description: String,
}

pub struct PppoeRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> PppoeRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> StorageResult<Vec<PppoeRow>> {
        Ok(sqlx::query_as::<_, PppoeRow>(
            r#"
            SELECT id, name, parent_iface, username, password, mtu,
                   default_route, keepalive, enabled, description
            FROM pppoe_profiles ORDER BY name
            "#,
        )
        .fetch_all(self.pool)
        .await?)
    }

    pub async fn find_by_name(&self, name: &str) -> StorageResult<Option<PppoeRow>> {
        Ok(sqlx::query_as::<_, PppoeRow>(
            r#"
            SELECT id, name, parent_iface, username, password, mtu,
                   default_route, keepalive, enabled, description
            FROM pppoe_profiles WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(self.pool)
        .await?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        &self,
        name: &str,
        parent_iface: &str,
        username: &str,
        password: &str,
        mtu: i32,
        default_route: bool,
        keepalive: i32,
        enabled: bool,
        description: &str,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO pppoe_profiles
              (name, parent_iface, username, password, mtu, default_route,
               keepalive, enabled, description, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9, NOW())
            ON CONFLICT (name) DO UPDATE SET
              parent_iface  = EXCLUDED.parent_iface,
              username      = EXCLUDED.username,
              password      = EXCLUDED.password,
              mtu           = EXCLUDED.mtu,
              default_route = EXCLUDED.default_route,
              keepalive     = EXCLUDED.keepalive,
              enabled       = EXCLUDED.enabled,
              description   = EXCLUDED.description,
              updated_at    = NOW()
            "#,
        )
        .bind(name)
        .bind(parent_iface)
        .bind(username)
        .bind(password)
        .bind(mtu)
        .bind(default_route)
        .bind(keepalive)
        .bind(enabled)
        .bind(description)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, name: &str) -> StorageResult<()> {
        let r = sqlx::query(r#"DELETE FROM pppoe_profiles WHERE name = $1"#)
            .bind(name)
            .execute(self.pool)
            .await?;
        if r.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.into()));
        }
        Ok(())
    }
}
