#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    /// از متغیر محیطی DATABASE_URL یا مقدار پیش‌فرض محلی
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://dezh:Dezh@127.0.0.1:5432/dezh".to_string()
        });
        Self { url }
    }
}