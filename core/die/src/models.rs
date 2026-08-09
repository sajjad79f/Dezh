use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub id: Uuid,
    pub subject: Uuid,
    pub message: String,
    pub severity: Severity,
}
