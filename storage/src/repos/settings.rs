use sqlx::PgPool;

use crate::error::StorageResult;

/// مخزن کلید/مقدار عمومی روی جدول `firewall_settings`.
/// آینده: می‌تواند برای تنظیمات سایر ماژول‌ها هم استفاده شود.
pub struct SettingsRepo<'a> {
    pool: &'a PgPool,
}

impl<'a> SettingsRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &str) -> StorageResult<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as(r#"SELECT value FROM firewall_settings WHERE key = $1"#)
                .bind(key)
                .fetch_optional(self.pool)
                .await?;
        Ok(row.map(|(v,)| v))
    }

    /// upsert — مقدار جدید جای مقدار قبلی را می‌گیرد
    pub async fn set(&self, key: &str, value: &str) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO firewall_settings (key, value, updated_at)
            VALUES ($1, $2, now())
            ON CONFLICT (key) DO UPDATE
            SET value = $2, updated_at = now()
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}