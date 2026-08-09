use uuid::Uuid;

use super::Severity;

#[derive(Debug, Clone)]
pub struct Finding {
    pub id: Uuid,
    pub subject: Uuid,
    pub message: String,
    pub severity: Severity,
}

impl Finding {
    pub fn new(subject: Uuid, message: impl Into<String>, severity: Severity) -> Self {
        Self {
            id: Uuid::new_v4(),
            subject,
            message: message.into(),
            severity,
        }
    }
}