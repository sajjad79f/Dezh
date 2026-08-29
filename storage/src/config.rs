#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            eprintln!(
                "[storage] DATABASE_URL not set — using local default (dev only)"
            );
            "postgres://dezh:Dezh@127.0.0.1:5432/dezh".to_string()
        });
        Self { url }
    }
}