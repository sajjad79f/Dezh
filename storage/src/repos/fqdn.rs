use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{StorageError, StorageResult};

/// یک شیء FQDN — دامنه‌ای که در قوانین به‌جای IP ثابت استفاده می‌شود.
/// رزولوشن را ترد پس‌زمینه dezh-fqdn انجام می‌دهد و نتیجه (IPهای v4/v6)
/// اینجا کش می‌شود؛ قوانین به setهای nft (dezh_fqdn_{name}_4/_6) ارجاع
/// می‌دهند، نه مستقیم به دامنه.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FqdnObjectRow {
    pub id: Uuid,
    pub name: String,
    /// دامنه واردشده — فقط `example.com` یا `*.example.com` مجاز است
    pub fqdn: String,
    pub description: String,
    pub enabled: bool,
    /// آخرین رزولوشن موفق — IPv4 و IPv6 مخلوط (تفکیک هنگام apply)؛
    /// در شکست موقت دست‌نخورده می‌ماند (fail-open)
    pub resolved: Vec<String>,
    pub last_resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    /// آخرین خطای رزولوشن — جدا از resolved تا کش سالم حفظ شود
    pub last_error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl FqdnObjectRow {
    /// دامنه‌ای که باید resolve شود — از wildcard فقط `*.` مجاز است و
    /// همیشه دامنه پایه resolve می‌شود، نه خود `*.`
    pub fn base_domain(&self) -> &str {
        self.fqdn.strip_prefix("*.").unwrap_or(self.fqdn.as_str()).trim()
    }

    /// آیا الگوی واردشده wildcard است؟
    pub fn is_wildcard(&self) -> bool {
        self.fqdn.starts_with("*.")
    }
}

pub type FqdnRepo<'a> = FqdnObjectRepo<'a>;
pub struct FqdnObjectRepo<'a> {
    pool: &'a PgPool,
}

const FQDN_COLUMNS: &str = "id, name, fqdn, description, enabled, \
                            resolved, last_resolved_at, last_error, \
                            created_at, updated_at";

impl<'a> FqdnObjectRepo<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, obj: &FqdnObjectRow) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO fqdn_objects (id, name, fqdn, description, enabled)
            VALUES ($1,$2,$3,$4,$5)
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .bind(obj.id)
        .bind(&obj.name)
        .bind(&obj.fqdn)
        .bind(&obj.description)
        .bind(obj.enabled)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn list(&self) -> StorageResult<Vec<FqdnObjectRow>> {
        let rows = sqlx::query_as::<_, FqdnObjectRow>(&format!(
            r#"
            SELECT {FQDN_COLUMNS}
            FROM fqdn_objects
            ORDER BY name
            "#
        ))
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    /// فقط اشیاء فعال — ترد dezh-fqdn فقط این‌ها را resolve می‌کند
    pub async fn list_enabled(&self) -> StorageResult<Vec<FqdnObjectRow>> {
        let rows = sqlx::query_as::<_, FqdnObjectRow>(&format!(
            r#"
            SELECT {FQDN_COLUMNS}
            FROM fqdn_objects
            WHERE enabled
            ORDER BY name
            "#
        ))
        .fetch_all(self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_name(&self, name: &str) -> StorageResult<Option<FqdnObjectRow>> {
        let row = sqlx::query_as::<_, FqdnObjectRow>(&format!(
            r#"
            SELECT {FQDN_COLUMNS}
            FROM fqdn_objects
            WHERE name = $1
            "#
        ))
        .bind(name)
        .fetch_optional(self.pool)
        .await?;
        Ok(row)
    }

    /// نوشتن نتیجه رزولوشن — یک متد برای هر دو حالت:
    /// error = None    → موفق: کش جدید + پاک شدن last_error + مهر زمان
    /// error = Some(e) → شکست: همان IPهای قبلی بازنویسی می‌شود، فقط خطا
    /// ثبت می‌شود و last_resolved_at دست‌نخورده می‌ماند (fail-open)
    pub async fn set_resolved(
        &self,
        name: &str,
        ips: &[String],
        error: Option<&str>,
    ) -> StorageResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE fqdn_objects
            SET resolved = $2,
                last_error = $3,
                last_resolved_at = CASE WHEN $3 IS NULL THEN now() ELSE last_resolved_at END,
                updated_at = now()
            WHERE name = $1
            "#,
        )
        .bind(name)
        .bind(ips)
        .bind(error)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.to_string()));
        }
        Ok(())
    }

    /// ثبت خطای رزولوشن — کش قبلی دست‌نخورده می‌ماند (fail-open)
    pub async fn update_error(&self, name: &str, error: &str) -> StorageResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE fqdn_objects
            SET last_error = $2, updated_at = now()
            WHERE name = $1
            "#,
        )
        .bind(name)
        .bind(error)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.to_string()));
        }
        Ok(())
    }

    /// تغییر دامنه یک شیء
    pub async fn update_fqdn(&self, name: &str, fqdn: &str) -> StorageResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE fqdn_objects
            SET fqdn = $2, updated_at = now()
            WHERE name = $1
            "#,
        )
        .bind(name)
        .bind(fqdn)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.to_string()));
        }
        Ok(())
    }

    pub async fn set_enabled(&self, name: &str, enabled: bool) -> StorageResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE fqdn_objects
            SET enabled = $2, updated_at = now()
            WHERE name = $1
            "#,
        )
        .bind(name)
        .bind(enabled)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.to_string()));
        }
        Ok(())
    }

    pub async fn delete(&self, name: &str) -> StorageResult<()> {
        let result = sqlx::query(r#"DELETE FROM fqdn_objects WHERE name = $1"#)
            .bind(name)
            .execute(self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(name.to_string()));
        }
        Ok(())
    }
}