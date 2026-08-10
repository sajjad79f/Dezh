use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: String,
    pub enabled: bool,
}

pub struct UserRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> UserRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub fn hash_password(password: &str) -> StorageResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| StorageError::Config(format!("hash error: {e}")))?
            .to_string();
        Ok(hash)
    }

    pub fn verify_password(password: &str, password_hash: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(password_hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }

    pub async fn create(
        &self,
        username: &str,
        password: &str,
        display_name: Option<&str>,
        role: &str,
    ) -> StorageResult<Uuid> {
        let id = Uuid::new_v4();
        let password_hash = Self::hash_password(password)?;

        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, display_name, role)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(&password_hash)
        .bind(display_name)
        .bind(role)
        .execute(self.pool)
        .await?;

        Ok(id)
    }

    pub async fn find_by_username(&self, username: &str) -> StorageResult<Option<UserRow>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, password_hash, display_name, role, enabled
            FROM users WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list(&self) -> StorageResult<Vec<UserRow>> {
        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT id, username, password_hash, display_name, role, enabled
            FROM users ORDER BY username
            "#,
        )
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn count(&self) -> StorageResult<i64> {
        let (n,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM users"#)
            .fetch_one(self.pool)
            .await?;
        Ok(n)
    }

    /// اگر هیچ کاربری نباشد، admin اولیه می‌سازد
    pub async fn ensure_bootstrap_admin(
        &self,
        username: &str,
        password: &str,
    ) -> StorageResult<Option<Uuid>> {
        if self.count().await? > 0 {
            return Ok(None);
        }
        let id = self
            .create(username, password, Some("Administrator"), "admin")
            .await?;
        Ok(Some(id))
    }
}