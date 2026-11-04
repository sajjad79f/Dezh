//! فاز ۵ — مدل شیء دامنه (FQDN Object)
//!
//! قوانین فایروال با «نام شیء» به آن ارجاع می‌دهند (مثل source_set/dest_set)
//! و در لحظه‌ی apply، شیء به IPهای resolve شده باز می‌شود.

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FqdnObject {
    pub id: Uuid,
    pub name: String,
    /// دامنه؛ مثل example.com یا *.example.com
    pub domain: String,
    pub description: String,
    pub enabled: bool,
    /// آخرین IPهای resolve شده (v4/v6 به‌صورت رشته)
    pub resolved: Vec<String>,
    pub last_resolved_at: Option<DateTime<Utc>>,
    /// آخرین خطای resolve — رشته‌ی خالی یعنی سالم
    pub last_error: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FqdnObject {
    /// آیا این شیء wildcard دارد؟ (شروع با "*.")
    pub fn is_wildcard(&self) -> bool {
        self.domain.starts_with("*.")
    }

    /// دامنه‌ی پایه برای resolve — از wildcard، "*." برداشته می‌شود
    pub fn base_domain(&self) -> &str {
        self.domain.strip_prefix("*.").unwrap_or(&self.domain)
    }
}

/// اعتبارسنجی دامنه — بدون وابستگی به regex
/// الگو: ["*.".] label ("." label)* — هر label حداکثر ۶۳ کاراکتر [a-z0-9-]
pub fn validate_domain(domain: &str) -> Result<(), String> {
    let d = domain.trim().to_ascii_lowercase();
    let body = d.strip_prefix("*.").unwrap_or(&d);

    if body.is_empty() {
        return Err("دامنه خالی است".to_string());
    }
    if body.len() > 253 {
        return Err("دامنه بیش از حد طولانی است (حداکثر ۲۵۳ کاراکتر)".to_string());
    }
    if !body.contains('.') {
        return Err("دامنه باید حداقل یک نقطه داشته باشد (مثل example.com)".to_string());
    }
    for label in body.split('.') {
        if label.is_empty() {
            return Err("بخش خالی در دامنه — نقطه‌ی تکراری یا ابتدا/انتهای دامنه".to_string());
        }
        if label.len() > 63 {
            return Err("هر بخش دامنه حداکثر ۶۳ کاراکتر است".to_string());
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(format!(
                "کاراکتر غیرمجاز «{}» در دامنه — فقط حروف، رقم و خط تیره",
                d
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err("خط تیره در ابتدا یا انتهای بخش دامنه مجاز نیست".to_string());
        }
    }
    Ok(())
}

/// نرمال‌سازی: trim + lowercase
pub fn normalize_domain(domain: &str) -> String {
    domain.trim().to_ascii_lowercase()
}