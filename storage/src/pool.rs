use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::DatabaseConfig;
use crate::error::StorageResult;
use crate::repos::UserRepo;

pub async fn create_pool(config: &DatabaseConfig) -> StorageResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.url)
        .await?;
    Ok(pool)
}

/// بعد از ساخت pool: اگر کاربری نبود، admin بساز
pub async fn bootstrap_admin(pool: &PgPool) -> StorageResult<()> {
    let repo = UserRepo::new(pool);

    let password = std::env::var("DEZH_ADMIN_PASSWORD")
        .unwrap_or_else(|_| "Changeme!Admin1".to_string());

    match repo.ensure_bootstrap_admin("admin", &password).await? {
        Some(id) => {
            eprintln!("[storage] bootstrap admin created id={id} (user=admin)");
            eprintln!("[storage] change DEZH_ADMIN_PASSWORD in production!");
        }
        None => {
            eprintln!("[storage] admin already exists — skip bootstrap");
        }
    }
    Ok(())
}