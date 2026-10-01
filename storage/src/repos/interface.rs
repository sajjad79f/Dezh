use sqlx::PgPool;

use crate::error::StorageResult;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InterfaceRow {
    pub name: String,
    pub zone: String,
    pub enabled: bool,
    pub ipv4_mode: String,
    pub address_cidr: Option<String>,
    pub gateway: Option<String>,
    pub description: String,
}

pub struct InterfaceRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> InterfaceRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> StorageResult<Vec<InterfaceRow>> {
        Ok(sqlx::query_as::<_, InterfaceRow>(
            r#"
            SELECT name, zone, enabled, ipv4_mode, address_cidr, gateway, description
            FROM network_interfaces ORDER BY name
            "#,
        )
        .fetch_all(self.pool)
        .await?)
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

    pub async fn update_config(
        &self,
        name: &str,
        zone: &str,
        enabled: bool,
        ipv4_mode: &str,
        address_cidr: Option<&str>,
        gateway: Option<&str>,
        description: &str,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO network_interfaces
              (name, zone, enabled, ipv4_mode, address_cidr, gateway, description, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7, NOW())
            ON CONFLICT (name) DO UPDATE SET
              zone = EXCLUDED.zone,
              enabled = EXCLUDED.enabled,
              ipv4_mode = EXCLUDED.ipv4_mode,
              address_cidr = EXCLUDED.address_cidr,
              gateway = EXCLUDED.gateway,
              description = EXCLUDED.description,
              updated_at = NOW()
            "#,
        )
        .bind(name)
        .bind(zone)
        .bind(enabled)
        .bind(ipv4_mode)
        .bind(address_cidr)
        .bind(gateway)
        .bind(description)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}