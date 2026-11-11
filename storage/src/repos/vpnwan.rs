use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VpnWanRow {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub server: String,
    pub username: String,
    pub password: String,
    pub mtu: i32,
    pub default_route: bool,
    pub keepalive: i32,
    pub enabled: bool,
    pub description: String,
}

pub struct VpnWanRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> VpnWanRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> StorageResult<Vec<VpnWanRow>> {
        Ok(sqlx::query_as::<_, VpnWanRow>(
            r#"
            SELECT id, name, kind, server, username, password, mtu,
                   default_route, keepalive, enabled, description
            FROM vpn_connections ORDER BY name
            "#,
        )
        .fetch_all(self.pool)
        .await?)
    }

    pub async fn find_by_name(&self, name: &str) -> StorageResult<Option<VpnWanRow>> {
        Ok(sqlx::query_as::<_, VpnWanRow>(
            r#"
            SELECT id, name, kind, server, username, password, mtu,
                   default_route, keepalive, enabled, description
            FROM vpn_connections WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(self.pool)
        .await?)
    }

    pub async fn list_by_kind(&self, kind: &str) -> StorageResult<Vec<VpnWanRow>> {
        Ok(sqlx::query_as::<_, VpnWanRow>(
            r#"
            SELECT id, name, kind, server, username, password, mtu,
                   default_route, keepalive, enabled, description
            FROM vpn_connections WHERE kind = $1 ORDER BY name
            "#,
        )
        .bind(kind)
        .fetch_all(self.pool)
        .await?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert(
        &self,
        name: &str,
        kind: &str,
        server: &str,
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
            INSERT INTO vpn_connections
              (name, kind, server, username, password, mtu, default_route,
               keepalive, enabled, description, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10, NOW())
            ON CONFLICT (name) DO UPDATE SET
              kind          = EXCLUDED.kind,
              server        = EXCLUDED.server,
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
        .bind(kind)
        .bind(server)
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
        let r = sqlx::query(r#"DELETE FROM vpn_connections WHERE name = $1"#)
            .bind(name)
            .execute(self.pool)
            .await?;
        if r.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.into()));
        }
        Ok(())
    }
}
