use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageResult;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IdentityRow {
    pub id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub source: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountingSessionRow {
    pub id: Uuid,
    pub identity_id: Option<Uuid>,
    pub protocol: String,
    pub ip_address: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub bytes_in: i64,
    pub bytes_out: i64,
}

pub struct IdentityRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> IdentityRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        username: &str,
        display_name: Option<&str>,
        source: &str,
    ) -> StorageResult<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO identities (id, username, display_name, source)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(display_name)
        .bind(source)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    pub async fn list(&self) -> StorageResult<Vec<IdentityRow>> {
        let rows = sqlx::query_as::<_, IdentityRow>(
            r#"
            SELECT id, username, display_name, source, enabled
            FROM identities ORDER BY username
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_username(&self, username: &str) -> StorageResult<Option<IdentityRow>> {
        let row = sqlx::query_as::<_, IdentityRow>(
            r#"
            SELECT id, username, display_name, source, enabled
            FROM identities WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }
}

pub struct AccountingRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> AccountingRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn start_session(
        &self,
        identity_id: Option<Uuid>,
        protocol: &str,
        ip_address: Option<&str>,
    ) -> StorageResult<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO accounting_sessions (id, identity_id, protocol, ip_address)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id)
        .bind(identity_id)
        .bind(protocol)
        .bind(ip_address)
        .execute(self.pool)
        .await?;
        Ok(id)
    }

    pub async fn end_session(
        &self,
        session_id: Uuid,
        bytes_in: i64,
        bytes_out: i64,
        cause: Option<&str>,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            UPDATE accounting_sessions
            SET ended_at = NOW(),
                bytes_in = $2,
                bytes_out = $3,
                terminate_cause = $4
            WHERE id = $1 AND ended_at IS NULL
            "#,
        )
        .bind(session_id)
        .bind(bytes_in)
        .bind(bytes_out)
        .bind(cause)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn end_open_sessions_for_identity(
        &self,
        identity_id: Uuid,
        bytes_in: i64,
        bytes_out: i64,
        cause: Option<&str>,
    ) -> StorageResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE accounting_sessions
            SET ended_at = NOW(),
                bytes_in = $2,
                bytes_out = $3,
                terminate_cause = $4
            WHERE identity_id = $1 AND ended_at IS NULL
            "#,
        )
        .bind(identity_id)
        .bind(bytes_in)
        .bind(bytes_out)
        .bind(cause)
        .execute(self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn find_open_session(
        &self,
        identity_id: Uuid,
        protocol: &str,
    ) -> StorageResult<Option<Uuid>> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM accounting_sessions
            WHERE identity_id = $1 AND protocol = $2 AND ended_at IS NULL
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(identity_id)
        .bind(protocol)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|(id,)| id))
    }

    pub async fn list_active(&self) -> StorageResult<Vec<AccountingSessionRow>> {
        let rows = sqlx::query_as::<_, AccountingSessionRow>(
            r#"
            SELECT id, identity_id, protocol, ip_address, started_at, ended_at, bytes_in, bytes_out
            FROM accounting_sessions
            WHERE ended_at IS NULL
            ORDER BY started_at DESC
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }
}