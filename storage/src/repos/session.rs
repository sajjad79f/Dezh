use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageResult;
use crate::repos::user::UserRow;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub struct SessionRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> SessionRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        token: &str,
        ttl_hours: i64,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> StorageResult<Uuid> {
        let id = Uuid::new_v4();
        let token_hash = Self::hash_token(token);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(ttl_hours);

        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, token_hash, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(&token_hash)
        .bind(ip)
        .bind(user_agent)
        .bind(expires_at)
        .execute(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn find_user_by_token(&self, token: &str) -> StorageResult<Option<UserRow>> {
        let token_hash = Self::hash_token(token);

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT u.id, u.username, u.password_hash, u.display_name, u.role, u.enabled
            FROM user_sessions s
            JOIN users u ON u.id = s.user_id
            WHERE s.token_hash = $1
              AND s.revoked_at IS NULL
              AND s.expires_at > NOW()
              AND u.enabled = TRUE
            "#,
        )
        .bind(&token_hash)
        .fetch_optional(self.pool)
        .await?;

        Ok(row)
    }

    pub async fn revoke(&self, token: &str) -> StorageResult<()> {
        let token_hash = Self::hash_token(token);
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET revoked_at = NOW()
            WHERE token_hash = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(&token_hash)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}