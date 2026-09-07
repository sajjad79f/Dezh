use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::DatabaseConfig;
use crate::error::StorageResult;
use crate::repos::UserRepo;

#[derive(Clone)]
pub struct DbPool(pub PgPool);

impl DbPool {
    pub fn inner(&self) -> &PgPool {
        &self.0
    }
}

impl std::ops::Deref for DbPool {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn create_pool(config: &DatabaseConfig) -> StorageResult<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.url)
        .await?;
    Ok(DbPool(pool))
}

pub async fn bootstrap_admin(pool: &DbPool) -> StorageResult<()> {
    let repo = UserRepo::new(pool.inner());

    let password = std::env::var("DEZH_ADMIN_PASSWORD")
        .unwrap_or_else(|_| "Admin123!".to_string());

    eprintln!("[storage] ensuring bootstrap admin (user=admin)");

    match repo.ensure_bootstrap_admin("admin", &password).await {
        Ok(Some(id)) => {
            eprintln!("[storage] bootstrap admin CREATED id={id}");
            eprintln!("[storage] login with username=admin and DEZH_ADMIN_PASSWORD (default Admin123!)");
        }
        Ok(None) => {
            eprintln!("[storage] users already exist — skip bootstrap admin");
        }
        Err(e) => {
            eprintln!("[storage] bootstrap admin FAILED: {e}");
            return Err(e);
        }
    }
    Ok(())
}

/// اجرای خودکار migrationها — در استارتاپ (dcm) صدا زده می‌شود
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("src/migrations").run(pool).await
}